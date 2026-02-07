<script setup lang="ts">
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { SchemaFieldInput } from "../../ipc/v1"
import { createTableV1, unwrapEnvelope } from "../../lib/tauriClient"
import {
	createFieldDraft,
	type FieldDraft,
	fieldTypeOptions,
	isVectorType,
	toFieldInput,
} from "./explorerShared"

const props = defineProps<{
	show: boolean
}>()

const emit = defineEmits<(e: "update:show", value: boolean) => void>()

const { activeProfileId, connectionId, setError, setStatus, refreshTables, openTable } =
	useWorkspace()

const { execute: execCreateTable, isLoading: isCreatingTable } = useCommand("创建表失败")
const createTableName = ref("")
const createFields = ref<FieldDraft[]>([createFieldDraft()])

function addCreateField() {
	createFields.value = [...createFields.value, createFieldDraft()]
}

function removeCreateField(index: number) {
	createFields.value = createFields.value.filter((_, idx) => idx !== index)
}

async function submitCreateTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId) return

	const tableName = createTableName.value.trim()
	if (!tableName) {
		setError("请输入表名")
		return
	}
	const fields = createFields.value.map(toFieldInput).filter(Boolean) as SchemaFieldInput[]
	if (!fields.length) {
		setError("至少需要一个字段")
		return
	}
	const invalidVector = fields.find(
		(field) => field.dataType === "fixed_size_list_float32" && !field.vectorLength
	)
	if (invalidVector) {
		setError("向量列需要指定维度")
		return
	}
	await execCreateTable(async () => {
		unwrapEnvelope(await createTableV1(currentConnectionId, tableName, { fields }))
		setStatus(`已创建表 ${tableName}`)
		await refreshTables(profileId)
		await openTable(profileId, tableName)
		resetForm()
		close()
	})
}

function resetForm() {
	createTableName.value = ""
	createFields.value = [createFieldDraft()]
}

function close() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(visible) => {
		if (!visible) {
			resetForm()
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		:mask-closable="!isCreatingTable"
		:close-on-esc="!isCreatingTable"
		@update:show="emit('update:show', $event)"
	>
		<NCard
			size="small"
			title="创建表"
			class="w-[760px] max-w-[calc(100vw-40px)]"
			:closable="!isCreatingTable"
			:bordered="false"
			@close="close"
		>
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-2">
					<label class="text-sm font-medium text-slate-600">表名</label>
					<NInput v-model:value="createTableName" placeholder="new_table" />
				</div>
				<div class="xl:col-span-4 flex items-end justify-end gap-2">
					<NButton quaternary :disabled="isCreatingTable" @click="close">
						取消
					</NButton>
					<NButton secondary :disabled="isCreatingTable" @click="addCreateField">
						添加字段
					</NButton>
					<NButton
						type="primary"
						:loading="isCreatingTable"
						@click="submitCreateTable"
					>
						创建表
					</NButton>
				</div>
			</div>

			<div class="mt-3 space-y-2">
				<div
					v-for="(field, index) in createFields"
					:key="`create-${index}`"
					class="grid gap-2 rounded-md border border-slate-100 bg-slate-50/60 p-2 md:grid-cols-12"
				>
					<NInput
						v-model:value="field.name"
						placeholder="字段名"
						class="md:col-span-4"
					/>
					<NSelect
						v-model:value="field.dataType"
						:options="fieldTypeOptions"
						class="md:col-span-4"
					/>
					<NCheckbox v-model:checked="field.nullable" class="md:col-span-2">
						可为空
					</NCheckbox>
					<NInputNumber
						v-if="isVectorType(field.dataType)"
						v-model:value="field.vectorLength"
						:min="1"
						placeholder="维度"
						class="md:col-span-2"
					/>
					<NButton
						v-if="createFields.length > 1"
						quaternary
						class="md:col-span-2"
						:disabled="isCreatingTable"
						@click="removeCreateField(index)"
					>
						移除
					</NButton>
				</div>
			</div>
		</NCard>
	</NModal>
</template>

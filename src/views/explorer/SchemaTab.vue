<script setup lang="ts">
import type { DataTableColumns } from "naive-ui"
import { computed, ref, watch } from "vue"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { FieldDataType, SchemaField, SchemaFieldInput } from "../../ipc/v1"
import { addColumnsV1, alterColumnsV1, dropColumnsV1, unwrapEnvelope } from "../../lib/tauriClient"
import {
	type AlterDraft,
	alterTypeOptions,
	type ColumnOpsTab,
	createAlterDraft,
	createFieldDraft,
	type FieldDraft,
	fieldTypeOptions,
	isVectorType,
	nullableOptions,
	renderHeader,
	resolveNullable,
	toFieldInput,
} from "./explorerShared"
import { formatSchemaDataType } from "./schemaTypes"

defineEmits<(e: "drop-table") => void>()

const { activeProfileId, activeTableId, schema, setError, setStatus, refreshSchema } =
	useWorkspace()

const hasActiveTable = computed(() => Boolean(activeTableId.value))

const schemaColumns: DataTableColumns<SchemaField> = [
	{ title: () => renderHeader("字段"), key: "name", ellipsis: { tooltip: true } },
	{
		title: () => renderHeader("类型"),
		key: "dataType",
		ellipsis: { tooltip: true },
		render: (row) => formatSchemaDataType(row.dataType),
	},
	{
		title: () => renderHeader("Nullable"),
		key: "nullable",
		ellipsis: { tooltip: true },
		render: (row) => (row.nullable ? "是" : "否"),
	},
]

const schemaData = computed(() => schema.value?.fields ?? [])
const allFieldNames = computed(() => schema.value?.fields.map((f) => f.name) ?? [])
const columnOptions = computed(() => allFieldNames.value.map((n) => ({ label: n, value: n })))

// ── Column operations ──────────────────────────────────

const showColumnOpsModal = ref(false)
const activeColumnOpsTab = ref<ColumnOpsTab>("add")

const addColumnFields = ref<FieldDraft[]>([createFieldDraft()])
const { execute: execAddColumns, isLoading: isAddingColumns } = useCommand("新增列失败")

const alterColumns = ref<AlterDraft[]>([createAlterDraft()])
const { execute: execAlterColumns, isLoading: isAlteringColumns } = useCommand("修改列失败")

const dropColumnNames = ref<string[]>([])
const { execute: execDropColumns, isLoading: isDroppingColumns } = useCommand("删除列失败")

const isColumnOpsBusy = computed(
	() => isAddingColumns.value || isAlteringColumns.value || isDroppingColumns.value
)

function openColumnOps(tab: ColumnOpsTab) {
	activeColumnOpsTab.value = tab
	showColumnOpsModal.value = true
}

function addColumnField() {
	addColumnFields.value = [...addColumnFields.value, createFieldDraft()]
}

function removeColumnField(index: number) {
	addColumnFields.value = addColumnFields.value.filter((_, idx) => idx !== index)
}

function addAlteration() {
	alterColumns.value = [...alterColumns.value, createAlterDraft()]
}

function removeAlteration(index: number) {
	alterColumns.value = alterColumns.value.filter((_, idx) => idx !== index)
}

async function submitAddColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId) {
		return
	}
	const fields = addColumnFields.value.map(toFieldInput).filter(Boolean) as SchemaFieldInput[]
	if (!fields.length) {
		setError("请填写要新增的列")
		return
	}
	const invalidVector = fields.find(
		(f) => f.dataType === "fixed_size_list_float32" && !f.vectorLength
	)
	if (invalidVector) {
		setError("向量列需要指定维度")
		return
	}
	await execAddColumns(async () => {
		unwrapEnvelope(await addColumnsV1(tableId, { fields }))
		setStatus("已新增列")
		await refreshSchema(profileId)
		addColumnFields.value = [createFieldDraft()]
	})
}

async function submitAlterColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId) {
		return
	}
	let invalidVector = false
	const payload = alterColumns.value
		.map((draft) => {
			const path = draft.path.trim()
			if (!path) {
				return null
			}
			const rename = draft.rename.trim()
			const nullable = resolveNullable(draft.nullable)
			const dataType = draft.dataType === "keep" ? undefined : draft.dataType
			const vectorLength =
				dataType === "fixed_size_list_float32" ? Number(draft.vectorLength ?? 0) : undefined
			if (!rename && nullable === undefined && !dataType) {
				return null
			}
			if (dataType === "fixed_size_list_float32" && (!vectorLength || vectorLength <= 0)) {
				invalidVector = true
				return null
			}
			return {
				path,
				rename: rename || undefined,
				nullable,
				dataType,
				vectorLength: vectorLength && vectorLength > 0 ? vectorLength : undefined,
			}
		})
		.filter(Boolean)
	if (invalidVector) {
		setError("向量列需要指定维度")
		return
	}
	if (!payload.length) {
		setError("请填写需要修改的列信息")
		return
	}
	await execAlterColumns(async () => {
		unwrapEnvelope(
			await alterColumnsV1({
				tableId,
				columns: payload as {
					path: string
					rename?: string
					nullable?: boolean
					dataType?: FieldDataType
					vectorLength?: number
				}[],
			})
		)
		setStatus("已更新列信息")
		await refreshSchema(profileId)
		alterColumns.value = [createAlterDraft()]
	})
}

async function submitDropColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId) {
		return
	}
	if (!dropColumnNames.value.length) {
		setError("请选择需要删除的列")
		return
	}
	await execDropColumns(async () => {
		unwrapEnvelope(await dropColumnsV1(tableId, dropColumnNames.value))
		setStatus("已删除列")
		await refreshSchema(profileId)
		dropColumnNames.value = []
	})
}

// ── Reset on table switch ──────────────────────────────

watch(activeTableId, () => {
	addColumnFields.value = [createFieldDraft()]
	alterColumns.value = [createAlterDraft()]
	dropColumnNames.value = []
})
</script>

<template>
	<div>
		<NDataTable
			class="data-table"
			size="small"
			:columns="schemaColumns"
			:data="schemaData"
			:bordered="false"
		/>
		<div class="mt-4 flex flex-wrap items-center justify-between gap-2">
			<div class="flex items-center gap-2">
				<NPopconfirm
					positive-text="删除"
					negative-text="取消"
					@positive-click="$emit('drop-table')"
				>
					<template #trigger>
						<NButton
							type="error"
							secondary
							:disabled="!hasActiveTable"
						>
							删除表
						</NButton>
					</template>
					确定删除当前表吗？该操作不可撤销。
				</NPopconfirm>
			</div>
			<div class="flex items-center gap-2">
				<NButton secondary :disabled="!hasActiveTable" @click="openColumnOps('add')">
					新增列…
				</NButton>
				<NButton secondary :disabled="!hasActiveTable" @click="openColumnOps('alter')">
					修改列…
				</NButton>
				<NButton
					type="error"
					secondary
					:disabled="!hasActiveTable"
					@click="openColumnOps('drop')"
				>
					删除列…
				</NButton>
			</div>
		</div>

		<!-- Column operations modal -->
		<NModal
			v-model:show="showColumnOpsModal"
			:mask-closable="!isColumnOpsBusy"
			:close-on-esc="!isColumnOpsBusy"
		>
			<NCard
				size="small"
				title="列操作"
				class="w-[860px] max-w-[calc(100vw-40px)]"
				:closable="!isColumnOpsBusy"
				:bordered="false"
				@close="showColumnOpsModal = false"
			>
				<NTabs v-model:value="activeColumnOpsTab" type="line">
					<NTabPane name="add" tab="新增列">
						<div class="space-y-2">
							<div
								v-for="(field, index) in addColumnFields"
								:key="`add-${index}`"
								class="grid gap-2 rounded-md border border-slate-100 bg-slate-50/60 p-2 md:grid-cols-12"
							>
								<NInput
									v-model:value="field.name"
									placeholder="字段名"
									class="md:col-span-4"
									:disabled="isAddingColumns"
								/>
								<NSelect
									v-model:value="field.dataType"
									:options="fieldTypeOptions"
									class="md:col-span-4"
									:disabled="isAddingColumns"
								/>
								<NCheckbox
									v-model:checked="field.nullable"
									class="md:col-span-2"
									:disabled="isAddingColumns"
								>
									可为空
								</NCheckbox>
								<NInputNumber
									v-if="isVectorType(field.dataType)"
									v-model:value="field.vectorLength"
									:min="1"
									placeholder="维度"
									class="md:col-span-2"
									:disabled="isAddingColumns"
								/>
								<NButton
									v-if="addColumnFields.length > 1"
									quaternary
									class="md:col-span-2"
									:disabled="isAddingColumns"
									@click="removeColumnField(index)"
								>
									移除
								</NButton>
							</div>
							<div class="flex items-center justify-end gap-2">
								<NButton secondary :disabled="isAddingColumns" @click="addColumnField">
									添加列
								</NButton>
								<NButton type="primary" :loading="isAddingColumns" @click="submitAddColumns">
									提交新增
								</NButton>
							</div>
						</div>
					</NTabPane>

					<NTabPane name="alter" tab="修改列">
						<div class="space-y-2">
							<div
								v-for="(draft, index) in alterColumns"
								:key="`alter-${index}`"
								class="grid gap-2 rounded-md border border-slate-100 bg-slate-50/60 p-2 md:grid-cols-12"
							>
								<NInput
									v-model:value="draft.path"
									placeholder="列名路径"
									class="md:col-span-3"
									:disabled="isAlteringColumns"
								/>
								<NInput
									v-model:value="draft.rename"
									placeholder="新名称"
									class="md:col-span-3"
									:disabled="isAlteringColumns"
								/>
								<NSelect
									v-model:value="draft.nullable"
									:options="nullableOptions"
									class="md:col-span-2"
									:disabled="isAlteringColumns"
								/>
								<NSelect
									v-model:value="draft.dataType"
									:options="alterTypeOptions"
									class="md:col-span-2"
									:disabled="isAlteringColumns"
								/>
								<NInputNumber
									v-if="isVectorType(draft.dataType)"
									v-model:value="draft.vectorLength"
									:min="1"
									placeholder="维度"
									class="md:col-span-1"
									:disabled="isAlteringColumns"
								/>
								<NButton
									v-if="alterColumns.length > 1"
									quaternary
									class="md:col-span-1"
									:disabled="isAlteringColumns"
									@click="removeAlteration(index)"
								>
									移除
								</NButton>
							</div>
							<div class="flex items-center justify-end gap-2">
								<NButton secondary :disabled="isAlteringColumns" @click="addAlteration">
									添加修改
								</NButton>
								<NButton
									type="primary"
									:loading="isAlteringColumns"
									@click="submitAlterColumns"
								>
									提交修改
								</NButton>
							</div>
						</div>
					</NTabPane>

					<NTabPane name="drop" tab="删除列">
						<div class="space-y-2">
							<NSelect
								v-model:value="dropColumnNames"
								:options="columnOptions"
								multiple
								clearable
								placeholder="选择要删除的列"
								:disabled="isDroppingColumns"
							/>
							<div class="flex justify-end">
								<NPopconfirm
									positive-text="删除"
									negative-text="取消"
									@positive-click="submitDropColumns"
								>
									<template #trigger>
										<NButton
											type="error"
											secondary
											:loading="isDroppingColumns"
											:disabled="isDroppingColumns"
										>
											删除列
										</NButton>
									</template>
									确定删除选中的列吗？
								</NPopconfirm>
							</div>
						</div>
					</NTabPane>
				</NTabs>
			</NCard>
		</NModal>
	</div>
</template>

<style scoped>
.data-table :deep(.n-data-table-th),
.data-table :deep(.n-data-table-td) {
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.data-table :deep(.table-header-ellipsis) {
	display: inline-block;
	max-width: 100%;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	vertical-align: bottom;
}
</style>

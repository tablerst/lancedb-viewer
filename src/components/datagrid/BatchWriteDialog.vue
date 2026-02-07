<script setup lang="ts">
/**
 * BatchWriteDialog — modal dialog for batch writing rows.
 */
import type { WriteDataMode } from "../../ipc/v1"
import { writeModeOptions } from "../../views/explorer/explorerShared"

const props = defineProps<{
	show: boolean
	loading: boolean
}>()

const emit = defineEmits<{
	(e: "update:show", value: boolean): void
	(e: "submit", rows: unknown[], mode: WriteDataMode): void
}>()

const writeMode = ref<WriteDataMode>("append")
const writeRowsText = ref("[]")

function parseRows(): unknown[] | null {
	try {
		const parsed = JSON.parse(writeRowsText.value.trim())
		if (!Array.isArray(parsed)) return null
		return parsed
	} catch {
		return null
	}
}

function handleSubmit() {
	const rows = parseRows()
	if (!rows || rows.length === 0) return
	emit("submit", rows, writeMode.value)
}

function handleClose() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(v) => {
		if (v) {
			writeRowsText.value = "[]"
			writeMode.value = "append"
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		preset="card"
		title="批量写入数据"
		:bordered="false"
		style="width: 600px; max-width: 90vw"
		@update:show="handleClose"
	>
		<div class="space-y-4">
			<div class="flex items-end gap-3">
				<div class="w-40">
					<label class="mb-1 block text-sm font-medium text-slate-600">写入模式</label>
					<NSelect v-model:value="writeMode" :options="writeModeOptions" size="small" />
				</div>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium text-slate-600">JSON 行数据</label>
				<NInput
					v-model:value="writeRowsText"
					type="textarea"
					:autosize="{ minRows: 6, maxRows: 14 }"
					placeholder='[{"id": 1, "text": "hello", "vector": [0.1, 0.2, 0.3]}]'
				/>
			</div>
		</div>
		<template #action>
			<div class="flex justify-end gap-2">
				<NButton @click="handleClose">取消</NButton>
				<NButton type="primary" :loading="loading" @click="handleSubmit">
					写入
				</NButton>
			</div>
		</template>
	</NModal>
</template>

<script setup lang="ts">
import { inject, ref, watch } from "vue"
import { useWorkspace } from "../../composables/workspaceContext"
import type { WriteDataMode } from "../../ipc/v1"
import { deleteRowsV1, unwrapEnvelope, updateRowsV1, writeRowsV1 } from "../../lib/tauriClient"
import { TRIGGER_DATA_REFRESH_KEY, type UpdateDraft, writeModeOptions } from "./explorerShared"

const { activeProfileId, activeTableId, setError, setStatus, clearMessages } = useWorkspace()

const triggerDataRefresh = inject(TRIGGER_DATA_REFRESH_KEY, () => {})

// ── Write rows ─────────────────────────────────────────

const writeMode = ref<WriteDataMode>("append")
const writeRowsText = ref("[]")
const isWritingRows = ref(false)

function parseWriteRows(): unknown[] | null {
	try {
		const parsed = JSON.parse(writeRowsText.value.trim())
		if (!Array.isArray(parsed)) {
			return null
		}
		return parsed
	} catch {
		return null
	}
}

async function submitWriteRows() {
	const tableId = activeTableId.value
	if (!activeProfileId.value || !tableId || isWritingRows.value) {
		return
	}
	const rows = parseWriteRows()
	if (!rows || rows.length === 0) {
		setError("请输入 JSON 数组格式的行数据")
		return
	}
	try {
		isWritingRows.value = true
		clearMessages()
		unwrapEnvelope(await writeRowsV1(tableId, rows, writeMode.value))
		setStatus(`已写入 ${rows.length} 行数据`)
		triggerDataRefresh()
	} catch (error) {
		setError(error instanceof Error ? error.message : "写入数据失败")
	} finally {
		isWritingRows.value = false
	}
}

// ── Update rows ────────────────────────────────────────

const updateFilter = ref("")
const updateColumns = ref<UpdateDraft[]>([{ column: "", expr: "" }])
const isUpdatingRows = ref(false)

function addUpdateColumn() {
	updateColumns.value = [...updateColumns.value, { column: "", expr: "" }]
}

function removeUpdateColumn(index: number) {
	updateColumns.value = updateColumns.value.filter((_, idx) => idx !== index)
}

async function submitUpdateRows() {
	const tableId = activeTableId.value
	if (!activeProfileId.value || !tableId || isUpdatingRows.value) {
		return
	}
	const updates = updateColumns.value
		.map((item) => ({ column: item.column.trim(), expr: item.expr.trim() }))
		.filter((item) => item.column && item.expr)
	if (!updates.length) {
		setError("请填写需要更新的列与表达式")
		return
	}
	try {
		isUpdatingRows.value = true
		clearMessages()
		unwrapEnvelope(
			await updateRowsV1({
				tableId,
				filter: updateFilter.value.trim() || undefined,
				updates,
			})
		)
		setStatus("更新操作已提交")
		triggerDataRefresh()
	} catch (error) {
		setError(error instanceof Error ? error.message : "更新数据失败")
	} finally {
		isUpdatingRows.value = false
	}
}

// ── Delete rows ────────────────────────────────────────

const deleteFilter = ref("")
const isDeletingRows = ref(false)

async function submitDeleteRows() {
	const tableId = activeTableId.value
	const filter = deleteFilter.value.trim()
	if (!activeProfileId.value || !tableId || isDeletingRows.value) {
		return
	}
	if (!filter) {
		setError("请输入删除条件")
		return
	}
	try {
		isDeletingRows.value = true
		clearMessages()
		unwrapEnvelope(await deleteRowsV1(tableId, filter))
		setStatus("删除操作已提交")
		triggerDataRefresh()
	} catch (error) {
		setError(error instanceof Error ? error.message : "删除数据失败")
	} finally {
		isDeletingRows.value = false
	}
}

// ── Reset on table switch ──────────────────────────────

watch(activeTableId, () => {
	writeRowsText.value = "[]"
	writeMode.value = "append"
	updateFilter.value = ""
	updateColumns.value = [{ column: "", expr: "" }]
	deleteFilter.value = ""
})
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="写入数据" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-2">
					<label class="text-sm font-medium text-slate-600">写入模式</label>
					<NSelect v-model:value="writeMode" :options="writeModeOptions" />
				</div>
				<div class="xl:col-span-4 flex items-end justify-end">
					<NButton type="primary" :loading="isWritingRows" @click="submitWriteRows">
						写入
					</NButton>
				</div>
			</div>
			<div class="mt-3">
				<label class="text-sm font-medium text-slate-600">JSON 行数据</label>
				<NInput
					v-model:value="writeRowsText"
					type="textarea"
					:autosize="{ minRows: 6, maxRows: 12 }"
					placeholder='[{"id": 1, "text": "hello", "vector": [0.1, 0.2, 0.3]}]'
				/>
			</div>
		</NCard>

		<NCard size="small" title="更新数据" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">过滤条件（可选）</label>
					<NInput v-model:value="updateFilter" placeholder="id = 1" />
				</div>
				<div class="xl:col-span-3 flex items-end justify-end">
					<NButton secondary :disabled="isUpdatingRows" @click="addUpdateColumn">
						添加更新列
					</NButton>
					<NButton type="primary" :loading="isUpdatingRows" @click="submitUpdateRows">
						提交更新
					</NButton>
				</div>
			</div>
			<div class="mt-3 space-y-2">
				<div
					v-for="(item, index) in updateColumns"
					:key="`update-${index}`"
					class="grid gap-2 rounded-md border border-slate-100 bg-slate-50/60 p-2 md:grid-cols-12"
				>
					<NInput
						v-model:value="item.column"
						placeholder="列名"
						class="md:col-span-4"
					/>
					<NInput
						v-model:value="item.expr"
						placeholder="表达式，例如 text || '_v2'"
						class="md:col-span-6"
					/>
					<NButton
						v-if="updateColumns.length > 1"
						quaternary
						class="md:col-span-2"
						@click="removeUpdateColumn(index)"
					>
						移除
					</NButton>
				</div>
			</div>
		</NCard>

		<NCard size="small" title="删除数据" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-4">
					<label class="text-sm font-medium text-slate-600">删除条件</label>
					<NInput v-model:value="deleteFilter" placeholder="id = 1" />
				</div>
				<div class="xl:col-span-2 flex items-end justify-end">
					<NPopconfirm
						positive-text="删除"
						negative-text="取消"
						@positive-click="submitDeleteRows"
					>
						<template #trigger>
							<NButton type="error" secondary :loading="isDeletingRows">
								删除数据
							</NButton>
						</template>
						确定删除符合条件的数据吗？
					</NPopconfirm>
				</div>
			</div>
		</NCard>
	</div>
</template>

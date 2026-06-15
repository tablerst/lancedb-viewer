<script setup lang="ts">
import BatchDeleteDialog from "../../components/datagrid/BatchDeleteDialog.vue"
import BatchUpdateDialog from "../../components/datagrid/BatchUpdateDialog.vue"
import BatchWriteDialog from "../../components/datagrid/BatchWriteDialog.vue"
import DataGrid from "../../components/datagrid/DataGrid.vue"
import { buildFilterExpression } from "../../components/datagrid/filterParser"
import type { DataGridChangeset, DataGridQueryParams } from "../../components/datagrid/types"
import { useDataGridColumns } from "../../components/datagrid/useDataGridColumns"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { WriteDataMode } from "../../ipc/v1"
import {
	deleteRowsV1,
	scanV1,
	unwrapEnvelope,
	updateRowsV1,
	writeRowsV1,
} from "../../lib/tauriClient"
import { DATA_REFRESH_KEY, TRIGGER_DATA_REFRESH_KEY } from "./explorerShared"

const emit = defineEmits<(e: "request-export") => void>()

const { activeProfileId, activeTableId, schema, setError, setStatus, clearMessages } =
	useWorkspace()

const dataRefreshTrigger = inject(DATA_REFRESH_KEY, ref(0))
const triggerDataRefresh = inject(TRIGGER_DATA_REFRESH_KEY, () => {})

// ── DataGrid columns from schema ───────────────────────

const { columns } = useDataGridColumns(schema)

// ── Browse state ───────────────────────────────────────

const hasActiveTable = computed(() => Boolean(activeTableId.value))

const limit = ref(50)
const offset = ref(0)
const globalFilter = ref("")

const isScanning = ref(false)
const scanError = ref("")
const dataRows = ref<Record<string, unknown>[]>([])
const nextOffset = ref<number | null>(null)
const loadTimeMs = ref<number | null>(null)
const showAdvancedFilter = ref(false)

// ── Batch operation modals ─────────────────────────────

const showBatchWrite = ref(false)
const showBatchUpdate = ref(false)
const showBatchDelete = ref(false)

// ── Scan ───────────────────────────────────────────────

async function runScan(queryParams?: Partial<DataGridQueryParams>) {
	const tableId = activeTableId.value
	if (!tableId || isScanning.value) return

	const scanOffset = queryParams?.offset ?? offset.value
	const scanLimit = queryParams?.limit ?? limit.value

	// Build filter from column filters + global filter
	let filter: string | undefined
	if (queryParams?.columnFilters && schema.value) {
		const gf = (queryParams.globalFilter ?? globalFilter.value).trim()
		filter = buildFilterExpression(queryParams.columnFilters, gf, schema.value.fields)
	} else {
		const gf = globalFilter.value.trim()
		filter = gf || undefined
	}

	try {
		isScanning.value = true
		scanError.value = ""
		const startTime = performance.now()
		const response = unwrapEnvelope(
			await scanV1({
				tableId,
				format: "json",
				filter,
				limit: scanLimit,
				offset: scanOffset,
			})
		)
		loadTimeMs.value = performance.now() - startTime
		if (response.chunk.format !== "json") {
			scanError.value = "当前仅支持 JSON 数据块"
			return
		}
		dataRows.value = response.chunk.rows as Record<string, unknown>[]
		nextOffset.value = response.nextOffset ?? null
		offset.value = scanOffset
		limit.value = scanLimit
		setStatus(`已加载 ${response.chunk.rows.length} 行数据`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "扫描数据失败"
		scanError.value = message
		setError(message)
	} finally {
		isScanning.value = false
	}
}

// When switching tabs, this component is mounted with an already-selected table.
// In that case `activeTableId` doesn't change, so the watcher below won't run.
// Trigger an initial scan on mount to avoid a "no response" empty view.
onMounted(() => {
	if (activeTableId.value) {
		void runScan()
	}
})

// ── DataGrid event handlers ────────────────────────────

function handleQuery(params: DataGridQueryParams) {
	void runScan(params)
}

function handleRefresh() {
	void runScan()
}

async function handleSave(changeset: DataGridChangeset) {
	const tableId = activeTableId.value
	if (!tableId) return

	try {
		// Process updates
		for (const update of changeset.updated) {
			unwrapEnvelope(
				await updateRowsV1({
					tableId,
					filter: update.filter,
					updates: update.updates,
				})
			)
		}

		// Process inserts
		if (changeset.inserted.length > 0) {
			unwrapEnvelope(await writeRowsV1(tableId, changeset.inserted, "append"))
		}

		// Process deletes
		for (const deleteFilter of changeset.deleted) {
			unwrapEnvelope(await deleteRowsV1({ tableId, filter: deleteFilter }))
		}

		const total = changeset.updated.length + changeset.inserted.length + changeset.deleted.length
		setStatus(`已保存 ${total} 项更改`)
		triggerDataRefresh()
	} catch (error) {
		const message = error instanceof Error ? error.message : "保存更改失败"
		setError(message)
	}
}

function handleExport() {
	emit("request-export")
}

function toggleAdvancedFilter() {
	showAdvancedFilter.value = !showAdvancedFilter.value
}

// ── Batch write ────────────────────────────────────────

const { execute: execWriteRows, isLoading: isWritingRows } = useCommand("写入数据失败")

async function handleBatchWrite(rows: unknown[], mode: WriteDataMode) {
	const tableId = activeTableId.value
	if (!activeProfileId.value || !tableId) return
	await execWriteRows(async () => {
		unwrapEnvelope(await writeRowsV1(tableId, rows, mode))
		setStatus(`已写入 ${rows.length} 行数据`)
		showBatchWrite.value = false
		triggerDataRefresh()
	})
}

// ── Batch update ───────────────────────────────────────

const { execute: execUpdateRows, isLoading: isUpdatingRows } = useCommand("更新数据失败")

async function handleBatchUpdate(
	filter: string | undefined,
	updates: Array<{ column: string; expr: string }>
) {
	const tableId = activeTableId.value
	if (!activeProfileId.value || !tableId) return
	await execUpdateRows(async () => {
		unwrapEnvelope(await updateRowsV1({ tableId, filter, updates }))
		setStatus("更新操作已提交")
		showBatchUpdate.value = false
		triggerDataRefresh()
	})
}

// ── Batch delete ───────────────────────────────────────

const { execute: execDeleteRows, isLoading: isDeletingRows } = useCommand("删除数据失败")

async function handleBatchDelete(filter: string) {
	const tableId = activeTableId.value
	if (!activeProfileId.value || !tableId) return
	await execDeleteRows(async () => {
		unwrapEnvelope(await deleteRowsV1({ tableId, filter }))
		setStatus("删除操作已提交")
		showBatchDelete.value = false
		triggerDataRefresh()
	})
}

// ── Watchers ───────────────────────────────────────────

watch(activeTableId, () => {
	offset.value = 0
	dataRows.value = []
	nextOffset.value = null
	scanError.value = ""
	loadTimeMs.value = null
	clearMessages()
	globalFilter.value = ""
	if (activeTableId.value) {
		void runScan()
	}
})

watch(dataRefreshTrigger, () => {
	if (activeTableId.value) {
		void runScan()
	}
})
</script>

<template>
	<div class="flex h-full flex-col">
		<!-- DataGrid (browse + inline edit) -->
		<DataGrid
			:columns="columns"
			:rows="dataRows"
			:loading="isScanning"
			:offset="offset"
			:limit="limit"
			:has-next-page="nextOffset !== null"
			:editable="hasActiveTable"
			:error="scanError"
			:show-advanced-filter="showAdvancedFilter"
			:global-filter="globalFilter"
			:load-time-ms="loadTimeMs"
			@query="handleQuery"
			@save="handleSave"
			@refresh="handleRefresh"
			@export="handleExport"
			@toggle-advanced-filter="toggleAdvancedFilter"
			@open-batch-write="showBatchWrite = true"
			@open-batch-update="showBatchUpdate = true"
			@open-batch-delete="showBatchDelete = true"
		>
			<!-- Advanced filter slot -->
			<template v-if="showAdvancedFilter" #advanced-filter>
				<div class="border-b border-slate-200 bg-slate-50/80 px-3 py-2">
					<div class="flex items-end gap-2">
						<div class="min-w-0 flex-1 space-y-1">
							<label class="text-xs font-medium text-slate-500">SQL 过滤表达式</label>
							<NInput
								v-model:value="globalFilter"
								size="small"
								placeholder='例如: id > 5 AND name LIKE "test%"'
								:disabled="isScanning || !hasActiveTable"
								@keydown.enter="handleRefresh"
							/>
						</div>
					</div>
				</div>
			</template>
		</DataGrid>

		<!-- Batch operation modals -->
		<BatchWriteDialog
			v-model:show="showBatchWrite"
			:loading="isWritingRows"
			@submit="handleBatchWrite"
		/>
		<BatchUpdateDialog
			v-model:show="showBatchUpdate"
			:loading="isUpdatingRows"
			@submit="handleBatchUpdate"
		/>
		<BatchDeleteDialog
			v-model:show="showBatchDelete"
			:loading="isDeletingRows"
			@submit="handleBatchDelete"
		/>
	</div>
</template>

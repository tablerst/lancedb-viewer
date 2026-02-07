<script setup lang="ts">
import { type DataTableColumns, NButton, NInput, NSpace, type SelectOption } from "naive-ui"
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from "vue"
import { useWorkspace } from "../../composables/workspaceContext"
import { formatCellValue, normalizeRow, renderCellValue } from "../../lib/formatters"
import { scanV1, unwrapEnvelope } from "../../lib/tauriClient"
import { compareValues, DATA_REFRESH_KEY, renderHeader } from "./explorerShared"

const { activeTableId, schema, setError, setStatus, clearMessages } = useWorkspace()

const dataRefreshTrigger = inject(DATA_REFRESH_KEY, ref(0))

const hasActiveTable = computed(() => Boolean(activeTableId.value))
const allFieldNames = computed(() => schema.value?.fields.map((f) => f.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((n) => ({ label: n, value: n }))
)

const selectedColumns = ref<string[]>([])
const limit = ref(50)
const offset = ref(0)
const filterExpression = ref("")
const columnFilters = ref<Record<string, string>>({})

const isScanning = ref(false)
const scanError = ref("")
const dataRows = ref<unknown[]>([])
const nextOffset = ref<number | null>(null)

const visibleColumns = computed(() =>
	selectedColumns.value.length ? selectedColumns.value : allFieldNames.value
)
const page = computed(() => Math.max(1, Math.floor(offset.value / limit.value) + 1))
const pageCount = computed(() =>
	Math.max(1, nextOffset.value === null ? page.value : page.value + 1)
)

// ── Column filter helpers ──────────────────────────────

function setColumnFilter(columnKey: string, value: string) {
	columnFilters.value = { ...columnFilters.value, [columnKey]: value }
}

function clearColumnFilters() {
	columnFilters.value = {}
}

function getColumnFilterValue(columnKey: string) {
	const value = columnFilters.value[columnKey]?.trim()
	return value ? value : null
}

// ── Column definitions ─────────────────────────────────

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() =>
	visibleColumns.value.map((name) => {
		const filterValue = columnFilters.value[name] ?? ""
		return {
			title: () => renderHeader(name),
			key: name,
			ellipsis: { tooltip: true },
			sorter: (rowA, rowB) => compareValues(rowA[name], rowB[name]),
			render: (row) => renderCellValue(row[name]),
			filter: (value, row) => {
				if (value === null || value === undefined || value === "") {
					return true
				}
				const cellValue = formatCellValue(row[name])
				return cellValue.toLowerCase().includes(String(value).toLowerCase())
			},
			filterMultiple: false,
			filterOptionValue: getColumnFilterValue(name),
			renderFilter: ({ active, show }) =>
				h(
					"span",
					{
						class: ["table-filter-trigger", active || show ? "text-blue-600" : "text-slate-400"],
						title: "筛选",
					},
					"筛"
				),
			renderFilterMenu: ({ hide }) =>
				h("div", { class: "table-filter-menu" }, [
					h(NInput, {
						value: filterValue,
						placeholder: `筛选 ${name}`,
						clearable: true,
						size: "small",
						onUpdateValue: (v) => setColumnFilter(name, String(v ?? "")),
					}),
					h(
						NSpace,
						{ justify: "end", size: "small" },
						{
							default: () => [
								h(
									NButton,
									{
										size: "tiny",
										quaternary: true,
										onClick: () => {
											setColumnFilter(name, "")
											hide()
										},
									},
									{ default: () => "清除" }
								),
								h(NButton, { size: "tiny", onClick: () => hide() }, { default: () => "关闭" }),
							],
						}
					),
				]),
		}
	})
)

const tableData = computed(() =>
	dataRows.value.map((row, index) => ({
		__rowId: `${offset.value + index}`,
		...normalizeRow(row),
	}))
)

// ── Scan ───────────────────────────────────────────────

async function runScan() {
	const tableId = activeTableId.value
	if (!tableId || isScanning.value) {
		return
	}
	try {
		isScanning.value = true
		scanError.value = ""
		const response = unwrapEnvelope(
			await scanV1({
				tableId,
				format: "json",
				projection: selectedColumns.value.length ? selectedColumns.value : undefined,
				filter: filterExpression.value.trim() || undefined,
				limit: limit.value,
				offset: offset.value,
			})
		)
		if (response.chunk.format !== "json") {
			scanError.value = "当前仅支持 JSON 数据块"
			return
		}
		dataRows.value = response.chunk.rows
		nextOffset.value = response.nextOffset ?? null
		setStatus(`已加载 ${response.chunk.rows.length} 行数据`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "扫描数据失败"
		scanError.value = message
		setError(message)
	} finally {
		isScanning.value = false
	}
}

// ── Actions ────────────────────────────────────────────

function selectAllColumns() {
	selectedColumns.value = allFieldNames.value
}

function clearColumns() {
	selectedColumns.value = []
}

function handlePageChange(nextPage: number) {
	if (isScanning.value) {
		return
	}
	offset.value = Math.max(0, (nextPage - 1) * limit.value)
	void runScan()
}

function handlePageSizeChange(nextSize: number) {
	if (isScanning.value) {
		return
	}
	limit.value = nextSize
	offset.value = 0
	void runScan()
}

// ── Watchers ───────────────────────────────────────────

watch(schema, () => {
	selectedColumns.value = allFieldNames.value
})

watch(activeTableId, () => {
	offset.value = 0
	dataRows.value = []
	nextOffset.value = null
	scanError.value = ""
	clearMessages()
	clearColumnFilters()
	if (activeTableId.value) {
		void runScan()
	}
})

watch(dataRefreshTrigger, () => {
	if (activeTableId.value) {
		void runScan()
	}
})

// ── Keyboard shortcuts ─────────────────────────────────

function handleKeydown(event: KeyboardEvent) {
	if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
		event.preventDefault()
		if (hasActiveTable.value && !isScanning.value) {
			void runScan()
		}
	}
}

onMounted(() => window.addEventListener("keydown", handleKeydown))
onBeforeUnmount(() => window.removeEventListener("keydown", handleKeydown))
</script>

<template>
	<div class="data-tab-scroll min-h-[420px] max-h-[70vh] overflow-y-auto">
		<div class="sticky top-0 z-10 bg-white/95 px-3 py-2 backdrop-blur">
			<div class="grid gap-2 xl:grid-cols-6">
				<div class="space-y-1 xl:col-span-4">
					<label class="text-sm font-medium text-slate-600">过滤表达式</label>
					<NInput
						v-model:value="filterExpression"
						placeholder='only_if("id > 5")'
						:disabled="isScanning || !hasActiveTable"
					/>
				</div>
				<div class="flex items-end xl:col-span-2">
					<NButton
						type="primary"
						:loading="isScanning"
						:disabled="!hasActiveTable"
						@click="runScan"
					>
						查询
					</NButton>
				</div>
			</div>

			<div class="mt-2 flex items-end gap-2">
				<div class="min-w-0 flex-1 space-y-1">
					<label class="text-sm font-medium text-slate-600">列投影</label>
					<NSelect
						v-model:value="selectedColumns"
						:options="columnOptions"
						multiple
						clearable
						:disabled="isScanning || !hasActiveTable"
					/>
				</div>
				<NButton text :disabled="!hasActiveTable" @click="selectAllColumns">
					全选
				</NButton>
				<NButton text :disabled="!hasActiveTable" @click="clearColumns">
					清空
				</NButton>
			</div>
		</div>

		<div class="space-y-3 rounded-lg bg-slate-50/60 px-3 pb-3 pt-2">
			<NAlert v-if="scanError" type="error" :bordered="false">
				{{ scanError }}
			</NAlert>

			<div v-if="isScanning && !tableData.length" class="space-y-2 py-4">
				<NSkeleton text :repeat="8" class="w-full" />
			</div>
			<NDataTable
				v-else
				class="data-table"
				size="small"
				:columns="tableColumns"
				:data="tableData"
				:loading="isScanning"
				:bordered="false"
				:row-key="(row) => row.__rowId"
			/>
		</div>

		<div
			class="sticky bottom-0 z-10 flex items-center justify-between gap-2 border-t border-slate-100 bg-white/95 px-3 py-2 backdrop-blur"
		>
			<NPagination
				size="small"
				:page="page"
				:page-size="limit"
				:page-count="pageCount"
				:disabled="isScanning || !hasActiveTable"
				show-size-picker
				:page-sizes="[10, 20, 50, 100]"
				@update:page="handlePageChange"
				@update:page-size="handlePageSizeChange"
			/>
		</div>
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

.table-filter-trigger {
	margin-left: 6px;
	font-size: 12px;
	line-height: 1;
	cursor: pointer;
}

.table-filter-menu {
	min-width: 220px;
	padding: 12px;
}
</style>

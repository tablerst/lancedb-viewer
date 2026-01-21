<script setup lang="ts">
import { NButton, NInput, NSpace, type DataTableColumns, type SelectOption } from "naive-ui"
import { computed, h, ref, watch } from "vue"
import { useWorkspace } from "../composables/workspaceContext"
import type { SchemaField, TableInfo } from "../ipc/v1"
import { formatCellValue, normalizeRow } from "../lib/formatters.ts"
import { scanV1, unwrapEnvelope } from "../lib/tauriClient"

const {
	activeProfileId,
	activeProfile,
	activeTableName,
	activeTableId,
	schema,
	isOpening,
	openTable,
	setError,
	setStatus,
	clearMessages,
} = useWorkspace()

const openedTables = ref<TableInfo[]>([])
const activeTableTab = ref<string | null>(null)
const columnFilters = ref<Record<string, string>>({})

function renderHeader(title: string) {
	return h("span", { class: "table-header-ellipsis", title }, title)
}

const schemaColumns: DataTableColumns<SchemaField> = [
	{ title: () => renderHeader("字段"), key: "name", ellipsis: { tooltip: true } },
	{ title: () => renderHeader("类型"), key: "dataType", ellipsis: { tooltip: true } },
	{
		title: () => renderHeader("Nullable"),
		key: "nullable",
		ellipsis: { tooltip: true },
		render: (row) => (row.nullable ? "是" : "否"),
	},
]

const schemaData = computed(() => schema.value?.fields ?? [])
const allFieldNames = computed(() => schema.value?.fields.map((field) => field.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((name) => ({ label: name, value: name }))
)

const selectedColumns = ref<string[]>([])
const limit = ref(50)
const offset = ref(0)
const filterExpression = ref("")

const isScanning = ref(false)
const scanError = ref("")
const dataRows = ref<unknown[]>([])
const nextOffset = ref<number | null>(null)
const hasOpenTables = computed(() => openedTables.value.length > 0)
const connectionLabel = computed(() => activeProfile.value?.name ?? "未连接")

const visibleColumns = computed(() =>
	selectedColumns.value.length ? selectedColumns.value : allFieldNames.value
)
const hasActiveTable = computed(() => Boolean(activeTableId.value))
const page = computed(() => Math.max(1, Math.floor(offset.value / limit.value) + 1))
const pageCount = computed(() =>
	Math.max(1, nextOffset.value === null ? page.value : page.value + 1)
)

function setColumnFilter(columnKey: string, value: string) {
	columnFilters.value = {
		...columnFilters.value,
		[columnKey]: value,
	}
}

function clearColumnFilters() {
	columnFilters.value = {}
}

function getColumnFilterValue(columnKey: string) {
	const value = columnFilters.value[columnKey]?.trim()
	return value ? value : null
}

function addOpenedTable(name: string) {
	if (!openedTables.value.some((table) => table.name === name)) {
		openedTables.value = [...openedTables.value, { name }]
	}
}

function getTabLabel(tableName: string) {
	return `${connectionLabel.value}-${tableName}`
}

function compareValues(a: unknown, b: unknown) {
	if (a === b) {
		return 0
	}
	if (a === null || a === undefined) {
		return -1
	}
	if (b === null || b === undefined) {
		return 1
	}
	const numA = typeof a === "number" ? a : Number(a)
	const numB = typeof b === "number" ? b : Number(b)
	if (!Number.isNaN(numA) && !Number.isNaN(numB)) {
		return numA - numB
	}
	return String(a).localeCompare(String(b))
}

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() =>
	visibleColumns.value.map((name) => {
		const filterValue = columnFilters.value[name] ?? ""
		return {
			title: () => renderHeader(name),
			key: name,
			ellipsis: { tooltip: true },
			sorter: (rowA, rowB) => compareValues(rowA[name], rowB[name]),
			render: (row) => formatCellValue(row[name]),
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
						class: [
							"table-filter-trigger",
							active || show ? "text-blue-600" : "text-slate-400",
						],
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
						onUpdateValue: (nextValue) =>
							setColumnFilter(name, String(nextValue ?? "")),
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
								h(
									NButton,
									{
										size: "tiny",
										onClick: () => hide(),
									},
									{ default: () => "关闭" }
								),
							],
						}
					)
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

watch(schema, () => {
	selectedColumns.value = allFieldNames.value
})

watch(activeProfileId, () => {
	openedTables.value = []
	activeTableTab.value = null
	clearColumnFilters()
})

watch(activeTableId, () => {
	offset.value = 0
	dataRows.value = []
	nextOffset.value = null
	scanError.value = ""
	clearMessages()
	clearColumnFilters()
	if (activeTableId.value && activeTableName.value) {
		addOpenedTable(activeTableName.value)
		activeTableTab.value = activeTableName.value
	}
	if (activeTableId.value) {
		void runScan()
	}
})

watch(activeTableTab, (nextTab) => {
	if (!nextTab || nextTab === activeTableName.value) {
		return
	}
	const profileId = activeProfileId.value
	if (!profileId) {
		return
	}
	void openTable(profileId, nextTab)
})

function selectAllColumns() {
	selectedColumns.value = allFieldNames.value
}

function clearColumns() {
	selectedColumns.value = []
}

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
</script>

<template>
	<div class="space-y-4">
		<NEmpty v-if="!hasOpenTables" description="选择表以查看详情" />
		<NTabs v-else v-model:value="activeTableTab" type="line">
			<NTabPane
				v-for="table in openedTables"
				:key="table.name"
				:name="table.name"
				:tab="getTabLabel(table.name)"
			>
				<template v-if="activeTableTab === table.name">
					<NTabs type="line">
						<NTabPane name="schema" tab="Schema">
							<NDataTable
								class="data-table"
								size="small"
								:columns="schemaColumns"
								:data="schemaData"
								:bordered="false"
							/>
						</NTabPane>
						<NTabPane name="data" tab="数据浏览">
							<div class="data-tab-scroll min-h-[420px] max-h-[70vh] overflow-y-auto">
								<div
									class="sticky top-0 z-10 bg-white/95 px-3 py-2 backdrop-blur"
								>
									<div class="flex flex-wrap items-center justify-between gap-2 text-sm text-slate-600">
										<span>当前表：{{ activeTableName }}</span>
										<span class="text-slate-400">•</span>
										<span v-if="isOpening">正在加载 schema…</span>
									</div>
									<div class="mt-2 grid gap-2 xl:grid-cols-5">
										<div class="space-y-1 xl:col-span-2">
											<label class="text-xs text-slate-500">过滤表达式</label>
											<NInput
												v-model:value="filterExpression"
												placeholder='only_if("id > 5")'
												:disabled="isScanning || !hasActiveTable"
											/>
										</div>
										<div class="flex flex-wrap items-end gap-2 xl:col-span-3">
											<NButton
												type="primary"
												:loading="isScanning"
												:disabled="!hasActiveTable"
												@click="runScan"
											>
												查询
											</NButton>
											<NButton
												secondary
												:disabled="!hasActiveTable"
												@click="selectAllColumns"
											>
												全部列
											</NButton>
											<NButton
												quaternary
												:disabled="!hasActiveTable"
												@click="clearColumns"
											>
												取消投影
											</NButton>
										</div>
									</div>

									<div class="mt-2 space-y-1">
										<label class="text-xs text-slate-500">列投影</label>
										<NSelect
											v-model:value="selectedColumns"
											:options="columnOptions"
											multiple
											clearable
											:disabled="isScanning || !hasActiveTable"
										/>
									</div>
								</div>

								<div class="space-y-3 px-3 pb-3 pt-2">
									<NAlert v-if="scanError" type="error" :bordered="false">
										{{ scanError }}
									</NAlert>

									<NDataTable
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
									<span class="text-xs text-slate-500">
										offset: {{ offset }} · limit: {{ limit }}
									</span>
								</div>
							</div>
						</NTabPane>
					</NTabs>
				</template>
			</NTabPane>
		</NTabs>
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

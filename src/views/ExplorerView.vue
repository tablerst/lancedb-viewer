<script setup lang="ts">
import { Menu, MenuItem } from "@tauri-apps/api/menu"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { confirm, open, save } from "@tauri-apps/plugin-dialog"
import { NButton, NInput, NSpace, type DataTableColumns, type SelectOption } from "naive-ui"
import { computed, h, ref, watch } from "vue"
import { useWorkspace } from "../composables/workspaceContext"
import type {
	DataFileFormatV1,
	FieldDataType,
	IndexDefinitionV1,
	IndexTypeV1,
	SchemaField,
	SchemaFieldInput,
	TableInfo,
	VersionInfoV1,
	WriteDataMode,
} from "../ipc/v1"
import { formatCellValue, formatTimestamp, normalizeRow } from "../lib/formatters.ts"
import {
	addColumnsV1,
	alterColumnsV1,
	checkoutTableLatestV1,
	checkoutTableVersionV1,
	cloneTableV1,
	createIndexV1,
	createTableV1,
	deleteRowsV1,
	dropIndexV1,
	dropColumnsV1,
	dropTableV1,
	exportDataV1,
	optimizeTableV1,
	getTableVersionV1,
	importDataV1,
	listIndexesV1,
	listVersionsV1,
	renameTableV1,
	scanV1,
	updateRowsV1,
	unwrapEnvelope,
	writeRowsV1,
} from "../lib/tauriClient"

const {
	activeProfileId,
	activeProfile,
	connectionId,
	activeTableName,
	activeTableId,
	schema,
	isOpening,
	openTable,
	refreshTables,
	refreshSchema,
	clearActiveTable,
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

const indexTypeLabels: Record<IndexTypeV1, string> = {
	auto: "Auto",
	btree: "BTree",
	bitmap: "Bitmap",
	label_list: "LabelList",
	fts: "FTS",
	ivf_flat: "IVF_FLAT",
	ivf_sq: "IVF_SQ",
	ivf_pq: "IVF_PQ",
	ivf_rq: "IVF_RQ",
	ivf_hnsw_pq: "IVF_HNSW_PQ",
	ivf_hnsw_sq: "IVF_HNSW_SQ",
}

const indexColumns: DataTableColumns<IndexDefinitionV1> = [
	{ title: () => renderHeader("索引名"), key: "name", ellipsis: { tooltip: true } },
	{
		title: () => renderHeader("类型"),
		key: "indexType",
		ellipsis: { tooltip: true },
		render: (row) => indexTypeLabels[row.indexType] ?? row.indexType,
	},
	{
		title: () => renderHeader("列"),
		key: "columns",
		ellipsis: { tooltip: true },
		render: (row) => row.columns.join(", "),
	},
]

const versionColumns: DataTableColumns<VersionInfoV1> = [
	{ title: () => renderHeader("版本"), key: "version", ellipsis: { tooltip: true } },
	{
		title: () => renderHeader("时间"),
		key: "timestamp",
		ellipsis: { tooltip: true },
		render: (row) => formatTimestamp(row.timestamp),
	},
	{
		title: () => renderHeader("元数据"),
		key: "metadata",
		ellipsis: { tooltip: true },
		render: (row) => formatMetadata(row.metadata),
	},
]

const schemaData = computed(() => schema.value?.fields ?? [])
const allFieldNames = computed(() => schema.value?.fields.map((field) => field.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((name) => ({ label: name, value: name }))
)
const indexNameOptions = computed<SelectOption[]>(() =>
	indexes.value.map((index) => ({ label: index.name, value: index.name }))
)

const selectedColumns = ref<string[]>([])
const limit = ref(50)
const offset = ref(0)
const filterExpression = ref("")
const activeInnerTab = ref("schema")

const canManageTables = computed(() => Boolean(connectionId.value))

type FieldDraft = {
	name: string
	dataType: FieldDataType
	nullable: boolean
	vectorLength?: number
}

type AlterDraft = {
	path: string
	rename: string
	nullable: "keep" | "nullable" | "not_nullable"
	dataType: "keep" | FieldDataType
	vectorLength?: number
}

const fieldTypeOptions: SelectOption[] = [
	{ label: "Int8", value: "int8" },
	{ label: "Int16", value: "int16" },
	{ label: "Int32", value: "int32" },
	{ label: "Int64", value: "int64" },
	{ label: "UInt8", value: "uint8" },
	{ label: "UInt16", value: "uint16" },
	{ label: "UInt32", value: "uint32" },
	{ label: "UInt64", value: "uint64" },
	{ label: "Float32", value: "float32" },
	{ label: "Float64", value: "float64" },
	{ label: "Boolean", value: "boolean" },
	{ label: "Utf8", value: "utf8" },
	{ label: "LargeUtf8", value: "large_utf8" },
	{ label: "Binary", value: "binary" },
	{ label: "LargeBinary", value: "large_binary" },
	{ label: "Vector(Float32)", value: "fixed_size_list_float32" },
]

const alterTypeOptions = computed<SelectOption[]>(() => [
	{ label: "保持不变", value: "keep" },
	...fieldTypeOptions,
])

const nullableOptions: SelectOption[] = [
	{ label: "保持不变", value: "keep" },
	{ label: "可为空", value: "nullable" },
	{ label: "不可为空", value: "not_nullable" },
]

const indexTypeOptions: SelectOption[] = [
	{ label: "Auto", value: "auto" },
	{ label: "BTree", value: "btree" },
	{ label: "Bitmap", value: "bitmap" },
	{ label: "LabelList", value: "label_list" },
	{ label: "FTS", value: "fts" },
	{ label: "IVF_FLAT", value: "ivf_flat" },
	{ label: "IVF_SQ", value: "ivf_sq" },
	{ label: "IVF_PQ", value: "ivf_pq" },
	{ label: "IVF_RQ", value: "ivf_rq" },
	{ label: "IVF_HNSW_PQ", value: "ivf_hnsw_pq" },
	{ label: "IVF_HNSW_SQ", value: "ivf_hnsw_sq" },
]

const createTableName = ref("")
const createFields = ref<FieldDraft[]>([createFieldDraft()])
const isCreatingTable = ref(false)

const addColumnFields = ref<FieldDraft[]>([createFieldDraft()])
const isAddingColumns = ref(false)

const alterColumns = ref<AlterDraft[]>([createAlterDraft()])
const isAlteringColumns = ref(false)

const dropColumnNames = ref<string[]>([])
const isDroppingColumns = ref(false)

const indexes = ref<IndexDefinitionV1[]>([])
const isLoadingIndexes = ref(false)
const indexError = ref("")
const indexType = ref<IndexTypeV1>("auto")
const indexColumnsToCreate = ref<string[]>([])
const indexName = ref("")
const indexReplace = ref(true)
const isCreatingIndex = ref(false)
const dropIndexName = ref("")
const isDroppingIndex = ref(false)

const versions = ref<VersionInfoV1[]>([])
const isLoadingVersions = ref(false)
const versionError = ref("")
const currentVersion = ref<number | null>(null)
const checkoutVersion = ref<number | null>(null)
const isCheckingOutVersion = ref(false)
const isCheckingOutLatest = ref(false)

const cloneTargetName = ref("")
const cloneSourceVersion = ref<number | null>(null)
const cloneIsShallow = ref(true)
const isCloningTable = ref(false)

const writeMode = ref<WriteDataMode>("append")
const writeRowsText = ref("[]")
const isWritingRows = ref(false)

type UpdateDraft = {
	column: string
	expr: string
}

const updateFilter = ref("")
const updateColumns = ref<UpdateDraft[]>([{ column: "", expr: "" }])
const isUpdatingRows = ref(false)

const deleteFilter = ref("")
const isDeletingRows = ref(false)

const writeModeOptions: SelectOption[] = [
	{ label: "追加写入", value: "append" },
	{ label: "覆盖写入", value: "overwrite" },
]

const fileFormatOptions: SelectOption[] = [
	{ label: "CSV", value: "csv" },
	{ label: "Parquet", value: "parquet" },
	{ label: "JSONL", value: "jsonl" },
]

const importFormat = ref<DataFileFormatV1>("csv")
const importPath = ref("")
const importMode = ref<WriteDataMode>("append")
const importHasHeader = ref(true)
const importDelimiter = ref(",")
const isImporting = ref(false)

const exportFormat = ref<DataFileFormatV1>("csv")
const exportPath = ref("")
const exportProjection = ref<string[]>([])
const exportFilter = ref("")
const exportLimit = ref<number | null>(1000)
const exportOffset = ref<number | null>(0)
const exportWithHeader = ref(true)
const exportDelimiter = ref(",")
const isExporting = ref(false)

const compactTargetRows = ref<number | null>(1_000_000)
const vacuumOlderThanDays = ref<number | null>(7)
const isCompacting = ref(false)
const isVacuuming = ref(false)

const isCsvImport = computed(() => importFormat.value === "csv")
const isCsvExport = computed(() => exportFormat.value === "csv")

const isScanning = ref(false)
const isDropping = ref(false)
const isRenamingTable = ref(false)
const renameTargetName = ref("")
const renameSourceTable = ref<string | null>(null)
const showRenameModal = ref(false)
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

function createFieldDraft(): FieldDraft {
	return {
		name: "",
		dataType: "utf8",
		nullable: true,
		vectorLength: 3,
	}
}

function createAlterDraft(): AlterDraft {
	return {
		path: "",
		rename: "",
		nullable: "keep",
		dataType: "keep",
		vectorLength: 3,
	}
}

function isVectorType(value: FieldDataType | "keep") {
	return value === "fixed_size_list_float32"
}

function toFieldInput(draft: FieldDraft): SchemaFieldInput | null {
	const name = draft.name.trim()
	if (!name) {
		return null
	}
	const input: SchemaFieldInput = {
		name,
		dataType: draft.dataType,
		nullable: draft.nullable,
	}
	if (isVectorType(draft.dataType)) {
		const length = Number(draft.vectorLength ?? 0)
		if (Number.isFinite(length) && length > 0) {
			input.vectorLength = length
		}
	}
	return input
}

function resolveNullable(value: AlterDraft["nullable"]): boolean | undefined {
	if (value === "keep") {
		return undefined
	}
	return value === "nullable"
}

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

function renderTableTabLabel(tableName: string) {
	return h(
		"span",
		{
			class: "table-tab-label",
			title: getTabLabel(tableName),
			onContextmenu: (event: MouseEvent) => {
				void showTableContextMenu(tableName, event)
			},
		},
		getTabLabel(tableName)
	)
}

function formatMetadata(metadata: Record<string, string>) {
	const entries = Object.entries(metadata ?? {})
	if (!entries.length) {
		return "—"
	}
	return entries.map(([key, value]) => `${key}=${value}`).join(", ")
}

const fileDialogFilters: Record<DataFileFormatV1, { name: string; extensions: string[] }> = {
	csv: { name: "CSV", extensions: ["csv"] },
	parquet: { name: "Parquet", extensions: ["parquet"] },
	jsonl: { name: "JSONL", extensions: ["jsonl", "json"] },
}

function resolveDialogPath(value: string | string[] | null) {
	if (!value) {
		return ""
	}
	if (Array.isArray(value)) {
		return value[0] ?? ""
	}
	return value
}

async function selectImportFile() {
	const selection = await open({
		multiple: false,
		filters: [fileDialogFilters[importFormat.value]],
	})
	const path = resolveDialogPath(selection)
	if (path) {
		importPath.value = path
	}
}

async function selectExportFile() {
	const selection = await save({
		filters: [fileDialogFilters[exportFormat.value]],
	})
	if (selection) {
		exportPath.value = selection
	}
}

async function ensureActiveTable(tableName: string) {
	const profileId = activeProfileId.value
	if (!profileId) {
		return false
	}
	if (activeTableName.value !== tableName) {
		await openTable(profileId, tableName)
	}
	activeTableTab.value = tableName
	return true
}

async function openRenameModal(tableName: string) {
	if (!(await ensureActiveTable(tableName))) {
		return
	}
	renameSourceTable.value = tableName
	renameTargetName.value = tableName
	showRenameModal.value = true
}

async function requestDropTable(tableName: string) {
	const shouldDrop = await confirm(`确定删除表 ${tableName} 吗？该操作不可撤销。`)
	if (!shouldDrop) {
		return
	}
	await dropTableByName(tableName)
}

async function showTableContextMenu(tableName: string, event: MouseEvent) {
	event.preventDefault()
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId) {
		return
	}
	const menu = await Menu.new({
		items: [
			await MenuItem.new({
				id: "open",
				text: "打开",
				action: async () => {
					await ensureActiveTable(tableName)
				},
			}),
			await MenuItem.new({
				id: "data",
				text: "数据浏览",
				action: async () => {
					if (await ensureActiveTable(tableName)) {
						activeInnerTab.value = "data"
					}
				},
			}),
			await MenuItem.new({
				id: "versions",
				text: "版本与时间旅行",
				action: async () => {
					if (await ensureActiveTable(tableName)) {
						activeInnerTab.value = "versions"
					}
				},
			}),
			await MenuItem.new({
				id: "rename",
				text: "重命名…",
				action: async () => {
					await openRenameModal(tableName)
				},
			}),
			await MenuItem.new({
				id: "drop",
				text: "删除表",
				action: async () => {
					await requestDropTable(tableName)
				},
			}),
		],
	})

	await menu.popup({
		window: getCurrentWindow(),
		position: { x: event.clientX, y: event.clientY },
	})
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
	createTableName.value = ""
	createFields.value = [createFieldDraft()]
	writeRowsText.value = "[]"
	writeMode.value = "append"
	importFormat.value = "csv"
	importPath.value = ""
	importMode.value = "append"
	importHasHeader.value = true
	importDelimiter.value = ","
	isImporting.value = false
	exportFormat.value = "csv"
	exportPath.value = ""
	exportProjection.value = []
	exportFilter.value = ""
	exportLimit.value = 1000
	exportOffset.value = 0
	exportWithHeader.value = true
	exportDelimiter.value = ","
	isExporting.value = false
	compactTargetRows.value = 1_000_000
	vacuumOlderThanDays.value = 7
	isCompacting.value = false
	isVacuuming.value = false
	updateFilter.value = ""
	updateColumns.value = [{ column: "", expr: "" }]
	deleteFilter.value = ""
	activeInnerTab.value = "schema"
	indexes.value = []
	indexError.value = ""
	indexType.value = "auto"
	indexColumnsToCreate.value = []
	indexName.value = ""
	indexReplace.value = true
	dropIndexName.value = ""
	isLoadingIndexes.value = false
	isCreatingIndex.value = false
	isDroppingIndex.value = false
	versions.value = []
	versionError.value = ""
	currentVersion.value = null
	checkoutVersion.value = null
	cloneTargetName.value = ""
	cloneSourceVersion.value = null
	cloneIsShallow.value = true
	isLoadingVersions.value = false
	isCheckingOutVersion.value = false
	isCheckingOutLatest.value = false
	isCloningTable.value = false
	isRenamingTable.value = false
	renameTargetName.value = ""
	renameSourceTable.value = null
	showRenameModal.value = false
	isLoadingVersions.value = false
	isCheckingOutVersion.value = false
	isCheckingOutLatest.value = false
	isCloningTable.value = false
	isRenamingTable.value = false
	renameTargetName.value = ""
	renameSourceTable.value = null
	showRenameModal.value = false
	clearColumnFilters()
})

watch(activeTableId, () => {
	offset.value = 0
	dataRows.value = []
	nextOffset.value = null
	scanError.value = ""
	clearMessages()
	clearColumnFilters()
	addColumnFields.value = [createFieldDraft()]
	alterColumns.value = [createAlterDraft()]
	dropColumnNames.value = []
	writeRowsText.value = "[]"
	writeMode.value = "append"
	importFormat.value = "csv"
	importPath.value = ""
	importMode.value = "append"
	importHasHeader.value = true
	importDelimiter.value = ","
	isImporting.value = false
	exportFormat.value = "csv"
	exportPath.value = ""
	exportProjection.value = []
	exportFilter.value = ""
	exportLimit.value = 1000
	exportOffset.value = 0
	exportWithHeader.value = true
	exportDelimiter.value = ","
	isExporting.value = false
	compactTargetRows.value = 1_000_000
	vacuumOlderThanDays.value = 7
	isCompacting.value = false
	isVacuuming.value = false
	updateFilter.value = ""
	updateColumns.value = [{ column: "", expr: "" }]
	deleteFilter.value = ""
	activeInnerTab.value = "schema"
	indexes.value = []
	indexError.value = ""
	indexType.value = "auto"
	indexColumnsToCreate.value = []
	indexName.value = ""
	indexReplace.value = true
	dropIndexName.value = ""
	versions.value = []
	versionError.value = ""
	currentVersion.value = null
	checkoutVersion.value = null
	cloneTargetName.value = ""
	cloneSourceVersion.value = null
	cloneIsShallow.value = true
	renameTargetName.value = ""
	isRenamingTable.value = false
	renameSourceTable.value = null
	showRenameModal.value = false
	if (activeTableId.value && activeTableName.value) {
		addOpenedTable(activeTableName.value)
		activeTableTab.value = activeTableName.value
	}
	if (activeTableId.value) {
		void runScan()
		void loadIndexes()
		void loadVersions()
		void loadCurrentVersion()
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

watch(showRenameModal, (visible) => {
	if (!visible) {
		renameSourceTable.value = null
		renameTargetName.value = ""
	}
})

function selectAllColumns() {
	selectedColumns.value = allFieldNames.value
}

function clearColumns() {
	selectedColumns.value = []
}

function addCreateField() {
	createFields.value = [...createFields.value, createFieldDraft()]
}

function removeCreateField(index: number) {
	createFields.value = createFields.value.filter((_, idx) => idx !== index)
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

function addUpdateColumn() {
	updateColumns.value = [...updateColumns.value, { column: "", expr: "" }]
}

function removeUpdateColumn(index: number) {
	updateColumns.value = updateColumns.value.filter((_, idx) => idx !== index)
}

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

async function submitCreateTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId || isCreatingTable.value) {
		return
	}
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

	try {
		isCreatingTable.value = true
		clearMessages()
		unwrapEnvelope(
			await createTableV1(currentConnectionId, tableName, { fields })
		)
		setStatus(`已创建表 ${tableName}`)
		await refreshTables(profileId)
		await openTable(profileId, tableName)
		createTableName.value = ""
		createFields.value = [createFieldDraft()]
	} catch (error) {
		const message = error instanceof Error ? error.message : "创建表失败"
		setError(message)
	} finally {
		isCreatingTable.value = false
	}
}

async function submitAddColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isAddingColumns.value) {
		return
	}
	const fields = addColumnFields.value.map(toFieldInput).filter(Boolean) as SchemaFieldInput[]
	if (!fields.length) {
		setError("请填写要新增的列")
		return
	}
	const invalidVector = fields.find(
		(field) => field.dataType === "fixed_size_list_float32" && !field.vectorLength
	)
	if (invalidVector) {
		setError("向量列需要指定维度")
		return
	}

	try {
		isAddingColumns.value = true
		clearMessages()
		unwrapEnvelope(await addColumnsV1(tableId, { fields }))
		setStatus("已新增列")
		await refreshSchema(profileId)
		addColumnFields.value = [createFieldDraft()]
	} catch (error) {
		const message = error instanceof Error ? error.message : "新增列失败"
		setError(message)
	} finally {
		isAddingColumns.value = false
	}
}

async function submitAlterColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isAlteringColumns.value) {
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
			const vectorLength = dataType === "fixed_size_list_float32"
				? Number(draft.vectorLength ?? 0)
				: undefined
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

	try {
		isAlteringColumns.value = true
		clearMessages()
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
	} catch (error) {
		const message = error instanceof Error ? error.message : "修改列失败"
		setError(message)
	} finally {
		isAlteringColumns.value = false
	}
}

async function submitDropColumns() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isDroppingColumns.value) {
		return
	}
	if (!dropColumnNames.value.length) {
		setError("请选择需要删除的列")
		return
	}

	try {
		isDroppingColumns.value = true
		clearMessages()
		unwrapEnvelope(await dropColumnsV1(tableId, dropColumnNames.value))
		setStatus("已删除列")
		await refreshSchema(profileId)
		dropColumnNames.value = []
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除列失败"
		setError(message)
	} finally {
		isDroppingColumns.value = false
	}
}

async function loadIndexes() {
	const tableId = activeTableId.value
	if (!tableId || isLoadingIndexes.value) {
		return
	}

	try {
		isLoadingIndexes.value = true
		indexError.value = ""
		const response = unwrapEnvelope(await listIndexesV1(tableId))
		indexes.value = response.indexes
	} catch (error) {
		const message = error instanceof Error ? error.message : "获取索引失败"
		indexError.value = message
		setError(message)
	} finally {
		isLoadingIndexes.value = false
	}
}

async function loadVersions() {
	const tableId = activeTableId.value
	if (!tableId || isLoadingVersions.value) {
		return
	}

	try {
		isLoadingVersions.value = true
		versionError.value = ""
		const response = unwrapEnvelope(await listVersionsV1({ tableId }))
		versions.value = response.versions
	} catch (error) {
		const message = error instanceof Error ? error.message : "获取版本列表失败"
		versionError.value = message
		setError(message)
	} finally {
		isLoadingVersions.value = false
	}
}

async function loadCurrentVersion() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}

	try {
		const response = unwrapEnvelope(await getTableVersionV1({ tableId }))
		currentVersion.value = response.version
	} catch (error) {
		const message = error instanceof Error ? error.message : "获取当前版本失败"
		setError(message)
	}
}

async function submitCreateIndex() {
	const tableId = activeTableId.value
	if (!tableId || isCreatingIndex.value) {
		return
	}
	const columns = indexColumnsToCreate.value
		.map((column) => column.trim())
		.filter(Boolean)
	if (!columns.length) {
		setError("请选择索引列")
		return
	}

	const name = indexName.value.trim() || undefined

	try {
		isCreatingIndex.value = true
		clearMessages()
		unwrapEnvelope(
			await createIndexV1({
				tableId,
				columns,
				indexType: indexType.value,
				name,
				replace: indexReplace.value,
			})
		)
		setStatus("索引创建已提交")
		await loadIndexes()
		indexName.value = ""
		indexColumnsToCreate.value = []
	} catch (error) {
		const message = error instanceof Error ? error.message : "创建索引失败"
		setError(message)
	} finally {
		isCreatingIndex.value = false
	}
}

async function submitDropIndex() {
	const tableId = activeTableId.value
	const indexNameValue = dropIndexName.value.trim()
	if (!tableId || isDroppingIndex.value) {
		return
	}
	if (!indexNameValue) {
		setError("请选择要删除的索引")
		return
	}

	try {
		isDroppingIndex.value = true
		clearMessages()
		unwrapEnvelope(await dropIndexV1(tableId, indexNameValue))
		setStatus(`已删除索引 ${indexNameValue}`)
		await loadIndexes()
		dropIndexName.value = ""
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除索引失败"
		setError(message)
	} finally {
		isDroppingIndex.value = false
	}
}

async function submitCheckoutVersion() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const version = checkoutVersion.value
	if (!profileId || !tableId || isCheckingOutVersion.value) {
		return
	}
	if (version === null) {
		setError("请输入版本号")
		return
	}
	if (version < 0) {
		setError("版本号不能为负数")
		return
	}

	try {
		isCheckingOutVersion.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await checkoutTableVersionV1({ tableId, version })
		)
		currentVersion.value = response.version
		setStatus(`已切换到版本 ${response.version}`)
		await refreshSchema(profileId)
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "切换版本失败"
		setError(message)
	} finally {
		isCheckingOutVersion.value = false
	}
}

async function submitCheckoutLatest() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isCheckingOutLatest.value) {
		return
	}

	try {
		isCheckingOutLatest.value = true
		clearMessages()
		const response = unwrapEnvelope(await checkoutTableLatestV1({ tableId }))
		currentVersion.value = response.version
		setStatus(`已回到最新版本 ${response.version}`)
		await refreshSchema(profileId)
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "恢复最新版本失败"
		setError(message)
	} finally {
		isCheckingOutLatest.value = false
	}
}

async function submitCloneTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableId = activeTableId.value
	if (!profileId || !currentConnectionId || !tableId || isCloningTable.value) {
		return
	}

	const targetName = cloneTargetName.value.trim()
	if (!targetName) {
		setError("请输入克隆表名")
		return
	}

	try {
		isCloningTable.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await cloneTableV1({
				connectionId: currentConnectionId,
				tableId,
				targetTableName: targetName,
				sourceVersion: cloneSourceVersion.value ?? undefined,
				isShallow: cloneIsShallow.value,
			})
		)
		setStatus(`已创建克隆表 ${response.name}`)
		cloneTargetName.value = ""
		cloneSourceVersion.value = null
		cloneIsShallow.value = true
		await refreshTables(profileId)
	} catch (error) {
		const message = error instanceof Error ? error.message : "克隆表失败"
		setError(message)
	} finally {
		isCloningTable.value = false
	}
}

async function submitWriteRows() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isWritingRows.value) {
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
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "写入数据失败"
		setError(message)
	} finally {
		isWritingRows.value = false
	}
}

async function submitUpdateRows() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isUpdatingRows.value) {
		return
	}
	const updates = updateColumns.value
		.map((item) => ({
			column: item.column.trim(),
			expr: item.expr.trim(),
		}))
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
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "更新数据失败"
		setError(message)
	} finally {
		isUpdatingRows.value = false
	}
}

async function submitDeleteRows() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const filter = deleteFilter.value.trim()
	if (!profileId || !tableId || isDeletingRows.value) {
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
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除数据失败"
		setError(message)
	} finally {
		isDeletingRows.value = false
	}
}

async function submitImportData() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const path = importPath.value.trim()
	if (!profileId || !tableId || isImporting.value) {
		return
	}
	if (!path) {
		setError("请选择要导入的文件")
		return
	}

	const delimiter = importDelimiter.value.trim()

	try {
		isImporting.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await importDataV1({
				tableId,
				path,
				format: importFormat.value,
				mode: importMode.value,
				hasHeader: importHasHeader.value,
				delimiter: delimiter ? delimiter : undefined,
			})
		)
		setStatus(`已导入 ${response.rows} 行数据`)
		await refreshSchema(profileId)
		offset.value = 0
		await runScan()
	} catch (error) {
		const message = error instanceof Error ? error.message : "导入失败"
		setError(message)
	} finally {
		isImporting.value = false
	}
}

async function submitExportData() {
	const tableId = activeTableId.value
	const path = exportPath.value.trim()
	if (!tableId || isExporting.value) {
		return
	}
	if (!path) {
		setError("请选择导出文件路径")
		return
	}

	const limit = exportLimit.value && exportLimit.value > 0 ? exportLimit.value : undefined
	const offsetValue = exportOffset.value ?? undefined
	const delimiter = exportDelimiter.value.trim()

	try {
		isExporting.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await exportDataV1({
				tableId,
				path,
				format: exportFormat.value,
				projection: exportProjection.value.length ? exportProjection.value : undefined,
				filter: exportFilter.value.trim() || undefined,
				limit,
				offset: offsetValue,
				delimiter: delimiter ? delimiter : undefined,
				withHeader: exportWithHeader.value,
			})
		)
		setStatus(`已导出 ${response.rows} 行数据到 ${response.path}`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "导出失败"
		setError(message)
	} finally {
		isExporting.value = false
	}
}

async function submitCompactTable() {
	const tableId = activeTableId.value
	if (!tableId || isCompacting.value) {
		return
	}
	const targetRows = compactTargetRows.value
	if (targetRows !== null && targetRows <= 0) {
		setError("目标片段行数必须大于 0")
		return
	}

	try {
		isCompacting.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await optimizeTableV1({
				tableId,
				action: "compact",
				targetRowsPerFragment: targetRows ?? undefined,
			})
		)
		setStatus(response.summary || "Compact 已完成")
	} catch (error) {
		const message = error instanceof Error ? error.message : "Compact 失败"
		setError(message)
	} finally {
		isCompacting.value = false
	}
}

async function submitVacuumTable() {
	const tableId = activeTableId.value
	if (!tableId || isVacuuming.value) {
		return
	}
	const olderThanDays = vacuumOlderThanDays.value
	if (olderThanDays !== null && olderThanDays < 0) {
		setError("保留天数不能为负数")
		return
	}

	try {
		isVacuuming.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await optimizeTableV1({
				tableId,
				action: "vacuum",
				olderThanDays: olderThanDays ?? undefined,
			})
		)
		setStatus(response.summary || "Vacuum 已完成")
	} catch (error) {
		const message = error instanceof Error ? error.message : "Vacuum 失败"
		setError(message)
	} finally {
		isVacuuming.value = false
	}
}

async function submitRenameTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableName = renameSourceTable.value ?? activeTableName.value
	if (!profileId || !currentConnectionId || !tableName || isRenamingTable.value) {
		return
	}

	const newTableName = renameTargetName.value.trim()
	if (!newTableName) {
		setError("请输入新表名")
		return
	}
	if (newTableName === tableName) {
		setError("新表名不能与当前表名相同")
		return
	}

	try {
		isRenamingTable.value = true
		clearMessages()
		const response = unwrapEnvelope(
			await renameTableV1({
				connectionId: currentConnectionId,
				tableName,
				newTableName,
			})
		)
		setStatus(`已重命名为 ${response.newTableName}`)
		renameTargetName.value = ""
		renameSourceTable.value = null
		showRenameModal.value = false
		openedTables.value = openedTables.value.map((table) =>
			table.name === tableName ? { name: response.newTableName } : table
		)
		activeTableTab.value = response.newTableName
		await refreshTables(profileId)
		await openTable(profileId, response.newTableName)
	} catch (error) {
		const message = error instanceof Error ? error.message : "重命名表失败"
		setError(message)
	} finally {
		isRenamingTable.value = false
	}
}

async function dropTableByName(tableName: string) {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId || !tableName || isDropping.value) {
		return
	}

	try {
		isDropping.value = true
		clearMessages()
		unwrapEnvelope(await dropTableV1(currentConnectionId, tableName))
		setStatus(`已删除表 ${tableName}`)

		openedTables.value = openedTables.value.filter((table) => table.name !== tableName)
		if (activeTableTab.value === tableName) {
			const nextTable = openedTables.value[0]?.name ?? null
			activeTableTab.value = nextTable
			if (!nextTable) {
				clearActiveTable(profileId)
			}
		}
		await refreshTables(profileId)
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除表失败"
		setError(message)
	} finally {
		isDropping.value = false
	}
}

async function dropActiveTable() {
	const tableName = activeTableName.value
	if (!tableName) {
		return
	}
	await dropTableByName(tableName)
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
		<NCard v-if="canManageTables" size="small" title="创建表" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-2">
					<label class="text-xs text-slate-500">表名</label>
					<NInput v-model:value="createTableName" placeholder="new_table" />
				</div>
				<div class="xl:col-span-4 flex items-end justify-end gap-2">
					<NButton
						secondary
						:disabled="isCreatingTable"
						@click="addCreateField"
					>
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
						@click="removeCreateField(index)"
					>
						移除
					</NButton>
				</div>
			</div>
		</NCard>

		<NEmpty v-else description="请先连接数据库" />
		<NEmpty v-if="canManageTables && !hasOpenTables" description="选择表以查看详情" />
		<NTabs v-else-if="hasOpenTables" v-model:value="activeTableTab" type="line">
			<NTabPane
				v-for="table in openedTables"
				:key="table.name"
				:name="table.name"
				:tab="renderTableTabLabel(table.name)"
			>
				<template v-if="activeTableTab === table.name">
					<NTabs v-model:value="activeInnerTab" type="line">
						<NTabPane name="schema" tab="Schema">
							<NDataTable
								class="data-table"
								size="small"
								:columns="schemaColumns"
								:data="schemaData"
								:bordered="false"
							/>
							<div class="mt-4 space-y-4">
								<NCard size="small" title="新增列" class="shadow-sm">
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
												v-if="addColumnFields.length > 1"
												quaternary
												class="md:col-span-2"
												@click="removeColumnField(index)"
											>
												移除
											</NButton>
										</div>
										<div class="flex items-center justify-end gap-2">
											<NButton
												secondary
												:disabled="isAddingColumns"
												@click="addColumnField"
											>
												添加列
											</NButton>
											<NButton
												type="primary"
												:loading="isAddingColumns"
												@click="submitAddColumns"
											>
												提交新增
											</NButton>
										</div>
									</div>
								</NCard>

								<NCard size="small" title="修改列" class="shadow-sm">
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
											/>
											<NInput
												v-model:value="draft.rename"
												placeholder="新名称"
												class="md:col-span-3"
											/>
											<NSelect
												v-model:value="draft.nullable"
												:options="nullableOptions"
												class="md:col-span-2"
											/>
											<NSelect
												v-model:value="draft.dataType"
												:options="alterTypeOptions"
												class="md:col-span-2"
											/>
											<NInputNumber
												v-if="isVectorType(draft.dataType)"
												v-model:value="draft.vectorLength"
												:min="1"
												placeholder="维度"
												class="md:col-span-1"
											/>
											<NButton
												v-if="alterColumns.length > 1"
												quaternary
												class="md:col-span-1"
												@click="removeAlteration(index)"
											>
												移除
											</NButton>
										</div>
										<div class="flex items-center justify-end gap-2">
											<NButton
												secondary
												:disabled="isAlteringColumns"
												@click="addAlteration"
											>
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
								</NCard>

								<NCard size="small" title="删除列" class="shadow-sm">
									<div class="space-y-2">
										<NSelect
											v-model:value="dropColumnNames"
											:options="columnOptions"
											multiple
											clearable
											placeholder="选择要删除的列"
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
													>
														删除列
													</NButton>
												</template>
												确定删除选中的列吗？
											</NPopconfirm>
										</div>
									</div>
								</NCard>
							</div>
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
											<NPopconfirm
												positive-text="删除"
												negative-text="取消"
												@positive-click="dropActiveTable"
											>
												<template #trigger>
													<NButton
														type="error"
														secondary
														:disabled="!hasActiveTable || isDropping"
														:loading="isDropping"
													>
														删除表
													</NButton>
												</template>
												确定删除当前表吗？该操作不可撤销。
											</NPopconfirm>
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
						<NTabPane name="write" tab="数据写入">
							<div class="space-y-4">
								<NCard size="small" title="写入数据" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-2">
											<label class="text-xs text-slate-500">写入模式</label>
											<NSelect
												v-model:value="writeMode"
												:options="writeModeOptions"
											/>
										</div>
										<div class="xl:col-span-4 flex items-end justify-end">
											<NButton
												type="primary"
												:loading="isWritingRows"
												@click="submitWriteRows"
											>
												写入
											</NButton>
										</div>
									</div>
									<div class="mt-3">
										<label class="text-xs text-slate-500">JSON 行数据</label>
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
											<label class="text-xs text-slate-500">过滤条件（可选）</label>
											<NInput v-model:value="updateFilter" placeholder="id = 1" />
										</div>
										<div class="xl:col-span-3 flex items-end justify-end">
											<NButton
												secondary
												:disabled="isUpdatingRows"
												@click="addUpdateColumn"
											>
												添加更新列
											</NButton>
											<NButton
												type="primary"
												:loading="isUpdatingRows"
												@click="submitUpdateRows"
											>
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
											<label class="text-xs text-slate-500">删除条件</label>
											<NInput v-model:value="deleteFilter" placeholder="id = 1" />
										</div>
										<div class="xl:col-span-2 flex items-end justify-end">
											<NPopconfirm
												positive-text="删除"
												negative-text="取消"
												@positive-click="submitDeleteRows"
											>
												<template #trigger>
													<NButton
														type="error"
														secondary
														:loading="isDeletingRows"
													>
														删除数据
													</NButton>
												</template>
												确定删除符合条件的数据吗？
											</NPopconfirm>
										</div>
									</div>
								</NCard>
							</div>
						</NTabPane>
						<NTabPane name="import-export" tab="导入导出">
							<div class="space-y-4">
								<NCard size="small" title="导入数据" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">文件路径</label>
											<div class="flex items-center gap-2">
												<NInput
													v-model:value="importPath"
													placeholder="选择要导入的文件"
													:disabled="isImporting"
												/>
												<NButton
													secondary
													:disabled="isImporting"
													@click="selectImportFile"
												>
													选择文件
												</NButton>
											</div>
										</div>
										<div class="xl:col-span-3 grid grid-cols-2 gap-3">
											<div>
												<label class="text-xs text-slate-500">格式</label>
												<NSelect
													v-model:value="importFormat"
													:options="fileFormatOptions"
													:disabled="isImporting"
												/>
											</div>
											<div>
												<label class="text-xs text-slate-500">写入模式</label>
												<NSelect
													v-model:value="importMode"
													:options="writeModeOptions"
													:disabled="isImporting"
												/>
											</div>
										</div>
									</div>
									<div class="mt-3 grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">CSV 选项</label>
											<div class="flex items-center gap-3">
												<NCheckbox
													v-model:checked="importHasHeader"
													:disabled="!isCsvImport || isImporting"
												>
													包含表头
												</NCheckbox>
												<NInput
													v-model:value="importDelimiter"
													placeholder="," 
													:disabled="!isCsvImport || isImporting"
													:maxlength="1"
													class="w-24"
												/>
											</div>
										</div>
										<div class="xl:col-span-3 flex items-end justify-end">
											<NButton
												type="primary"
												:loading="isImporting"
												:disabled="!hasActiveTable"
												@click="submitImportData"
											>
												开始导入
											</NButton>
										</div>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										导入将使用当前表 Schema，CSV/JSONL 列顺序需与表一致。
									</div>
								</NCard>

								<NCard size="small" title="导出数据" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">导出路径</label>
											<div class="flex items-center gap-2">
												<NInput
													v-model:value="exportPath"
													placeholder="选择导出文件位置"
													:disabled="isExporting"
												/>
												<NButton
													secondary
													:disabled="isExporting"
													@click="selectExportFile"
												>
													选择路径
												</NButton>
											</div>
										</div>
										<div class="xl:col-span-3 grid grid-cols-2 gap-3">
											<div>
												<label class="text-xs text-slate-500">格式</label>
												<NSelect
													v-model:value="exportFormat"
													:options="fileFormatOptions"
													:disabled="isExporting"
												/>
											</div>
											<div>
												<label class="text-xs text-slate-500">Filter</label>
												<NInput
													v-model:value="exportFilter"
													placeholder="id > 10"
													:disabled="isExporting"
												/>
											</div>
										</div>
									</div>
									<div class="mt-3 grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">列投影</label>
											<NSelect
												v-model:value="exportProjection"
												:options="columnOptions"
												multiple
												clearable
												:disabled="isExporting"
											/>
										</div>
										<div class="xl:col-span-3 grid grid-cols-3 gap-3">
											<div>
												<label class="text-xs text-slate-500">Limit</label>
												<NInputNumber
													v-model:value="exportLimit"
													:min="1"
													:disabled="isExporting"
												/>
											</div>
											<div>
												<label class="text-xs text-slate-500">Offset</label>
												<NInputNumber
													v-model:value="exportOffset"
													:min="0"
													:disabled="isExporting"
												/>
											</div>
											<div class="flex items-end justify-end">
												<NButton
													type="primary"
													:loading="isExporting"
													:disabled="!hasActiveTable"
													@click="submitExportData"
												>
													开始导出
												</NButton>
											</div>
										</div>
									</div>
									<div class="mt-3 flex flex-wrap items-center gap-3">
										<NCheckbox
											v-model:checked="exportWithHeader"
											:disabled="!isCsvExport || isExporting"
										>
											CSV 输出表头
										</NCheckbox>
										<NInput
											v-model:value="exportDelimiter"
											placeholder="," 
											:disabled="!isCsvExport || isExporting"
											:maxlength="1"
											class="w-24"
										/>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										Limit 清空表示不限制输出。
									</div>
								</NCard>
							</div>
						</NTabPane>
						<NTabPane name="maintenance" tab="维护">
							<div class="space-y-4">
								<NCard size="small" title="Compact（合并数据文件）" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">目标片段行数</label>
											<NInputNumber
												v-model:value="compactTargetRows"
												:min="1"
												:disabled="!hasActiveTable || isCompacting"
											/>
										</div>
										<div class="xl:col-span-3 flex items-end justify-end">
											<NButton
												type="primary"
												:loading="isCompacting"
												:disabled="!hasActiveTable"
												@click="submitCompactTable"
											>
												执行 Compact
											</NButton>
										</div>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										Compact 会合并小文件并重写片段，过程可能耗时。
									</div>
								</NCard>

								<NCard size="small" title="Vacuum（清理旧版本）" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">保留天数</label>
											<NInputNumber
												v-model:value="vacuumOlderThanDays"
												:min="0"
												:disabled="!hasActiveTable || isVacuuming"
											/>
										</div>
										<div class="xl:col-span-3 flex items-end justify-end">
											<NPopconfirm
												positive-text="执行"
												negative-text="取消"
												@positive-click="submitVacuumTable"
											>
												<template #trigger>
													<NButton
														type="primary"
														:loading="isVacuuming"
														:disabled="!hasActiveTable"
													>
														执行 Vacuum
													</NButton>
												</template>
												将清理超过指定天数的旧版本与未引用文件，确定继续吗？
											</NPopconfirm>
										</div>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										建议在低峰期执行；部分远程后端可能不支持该操作。
									</div>
								</NCard>
							</div>
						</NTabPane>
						<NTabPane name="versions" tab="版本与时间旅行">
							<div class="space-y-4">
								<NCard size="small" title="版本列表" class="shadow-sm">
									<div class="flex flex-wrap items-center justify-between gap-2 text-xs text-slate-500">
										<span>当前版本：{{ currentVersion ?? "—" }}</span>
										<div class="flex items-center gap-2">
											<NButton
												secondary
												:loading="isLoadingVersions"
												:disabled="!hasActiveTable"
												@click="loadVersions"
											>
												刷新版本
											</NButton>
											<NButton
												quaternary
												:disabled="!hasActiveTable"
												@click="loadCurrentVersion"
											>
												刷新当前版本
											</NButton>
										</div>
									</div>
									<NAlert v-if="versionError" type="error" :bordered="false" class="mt-3">
										{{ versionError }}
									</NAlert>
									<NDataTable
										class="data-table mt-3"
										size="small"
										:columns="versionColumns"
										:data="versions"
										:bordered="false"
										:row-key="(row) => row.version"
									/>
									<NEmpty v-if="!versions.length" description="暂无版本记录" class="mt-3" />
								</NCard>

								<NCard size="small" title="打开版本" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-2">
											<label class="text-xs text-slate-500">版本号</label>
											<NInputNumber
												v-model:value="checkoutVersion"
												:min="0"
												:disabled="!hasActiveTable"
											/>
										</div>
										<div class="xl:col-span-4 flex items-end justify-end gap-2">
											<NButton
												type="primary"
												:loading="isCheckingOutVersion"
												:disabled="!hasActiveTable"
												@click="submitCheckoutVersion"
											>
												打开版本
											</NButton>
											<NButton
												secondary
												:loading="isCheckingOutLatest"
												:disabled="!hasActiveTable"
												@click="submitCheckoutLatest"
											>
												回到最新
											</NButton>
										</div>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										切换版本后会刷新 Schema 与数据浏览。
									</div>
								</NCard>

								<NCard size="small" title="克隆/分支" class="shadow-sm">
									<div class="grid gap-3 xl:grid-cols-6">
										<div class="xl:col-span-3">
											<label class="text-xs text-slate-500">新表名</label>
											<NInput
												v-model:value="cloneTargetName"
												placeholder="clone_table"
												:disabled="!hasActiveTable"
											/>
										</div>
										<div class="xl:col-span-2">
											<label class="text-xs text-slate-500">源版本（可选）</label>
											<NInputNumber
												v-model:value="cloneSourceVersion"
												:min="0"
												placeholder="留空使用最新"
												:disabled="!hasActiveTable"
											/>
										</div>
										<div class="xl:col-span-1 flex items-end">
											<NCheckbox
												v-model:checked="cloneIsShallow"
												:disabled="!hasActiveTable"
											>
												浅克隆
											</NCheckbox>
										</div>
										<div class="xl:col-span-6 flex items-center justify-end">
											<NButton
												type="primary"
												:loading="isCloningTable"
												:disabled="!hasActiveTable"
												@click="submitCloneTable"
											>
												创建克隆
											</NButton>
										</div>
									</div>
									<div class="mt-2 text-xs text-slate-400">
										浅克隆共享数据文件，适合做分支试验；深克隆暂未实现。
									</div>
								</NCard>
							</div>
						</NTabPane>
							<NTabPane name="indexes" tab="索引管理">
								<div class="space-y-4">
									<NCard size="small" title="索引列表" class="shadow-sm">
										<div class="flex items-center justify-end">
											<NButton
												secondary
												:loading="isLoadingIndexes"
												:disabled="!hasActiveTable"
												@click="loadIndexes"
											>
												刷新索引
											</NButton>
										</div>
										<NAlert v-if="indexError" type="error" :bordered="false" class="mt-3">
											{{ indexError }}
										</NAlert>
										<NDataTable
											class="data-table mt-3"
											size="small"
											:columns="indexColumns"
											:data="indexes"
											:loading="isLoadingIndexes"
											:bordered="false"
										/>
										<NEmpty v-if="!indexes.length" description="暂无索引" class="mt-3" />
									</NCard>

									<NCard size="small" title="创建索引" class="shadow-sm">
										<div class="grid gap-3 xl:grid-cols-6">
											<div class="xl:col-span-2">
												<label class="text-xs text-slate-500">索引类型</label>
												<NSelect
													v-model:value="indexType"
													:options="indexTypeOptions"
													:disabled="!hasActiveTable"
												/>
											</div>
											<div class="xl:col-span-4">
												<label class="text-xs text-slate-500">索引列</label>
												<NSelect
													v-model:value="indexColumnsToCreate"
													:options="columnOptions"
													multiple
													clearable
													:disabled="!hasActiveTable"
												/>
											</div>
											<div class="xl:col-span-3">
												<label class="text-xs text-slate-500">索引名称（可选）</label>
												<NInput
													v-model:value="indexName"
													placeholder="my_index"
													:disabled="!hasActiveTable"
												/>
											</div>
											<div class="xl:col-span-3 flex items-end justify-end gap-3">
												<NCheckbox
													v-model:checked="indexReplace"
													:disabled="!hasActiveTable"
												>
													替换同名索引
												</NCheckbox>
												<NButton
													type="primary"
													:loading="isCreatingIndex"
													:disabled="!hasActiveTable"
													@click="submitCreateIndex"
												>
													创建索引
												</NButton>
											</div>
										</div>
									</NCard>

									<NCard size="small" title="删除索引" class="shadow-sm">
										<div class="grid gap-3 xl:grid-cols-6">
											<div class="xl:col-span-4">
												<label class="text-xs text-slate-500">选择索引</label>
												<NSelect
													v-model:value="dropIndexName"
													:options="indexNameOptions"
													clearable
													:disabled="!hasActiveTable"
												/>
											</div>
											<div class="xl:col-span-2 flex items-end justify-end">
												<NPopconfirm
													positive-text="删除"
													negative-text="取消"
													@positive-click="submitDropIndex"
												>
													<template #trigger>
														<NButton
															type="error"
															secondary
															:loading="isDroppingIndex"
															:disabled="!hasActiveTable"
														>
															删除索引
														</NButton>
													</template>
													确定删除选中的索引吗？
												</NPopconfirm>
											</div>
										</div>
									</NCard>
								</div>
							</NTabPane>
					</NTabs>
				</template>
			</NTabPane>
		</NTabs>

		<NModal v-model:show="showRenameModal">
			<NCard
				size="small"
				title="重命名表"
				class="w-[420px]"
				:closable="!isRenamingTable"
				:bordered="false"
			>
				<div class="space-y-3">
					<div class="text-xs text-slate-500">
						当前表：{{ renameSourceTable ?? activeTableName ?? "—" }}
					</div>
					<NInput
						v-model:value="renameTargetName"
						placeholder="new_table_name"
						:disabled="isRenamingTable"
					/>
					<div class="flex items-center justify-end gap-2">
						<NButton
							quaternary
							:disabled="isRenamingTable"
							@click="showRenameModal = false"
						>
							取消
						</NButton>
						<NButton
							type="primary"
							:loading="isRenamingTable"
							@click="submitRenameTable"
						>
							确认重命名
						</NButton>
					</div>
					<div class="text-[11px] text-slate-400">
						仅 LanceDB Cloud 支持重命名；本地连接将提示不支持。
					</div>
				</div>
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

.table-tab-label {
	cursor: context-menu;
}
</style>

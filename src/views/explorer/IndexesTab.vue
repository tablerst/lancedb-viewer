<script setup lang="ts">
import { Plus, RefreshCw, Trash2 } from "lucide-vue-next"
import { type DataTableColumns, NButton, NPopconfirm, NTag, type SelectOption } from "naive-ui"
import { computed, h, ref, watch } from "vue"

import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type {
	CreateIndexRequestV1,
	DistanceTypeV1,
	IndexDefinitionV1,
	IndexTypeV1,
} from "../../ipc/v1"
import { createIndexV1, dropIndexV1, listIndexesV1, unwrapEnvelope } from "../../lib/tauriClient"
import { indexTypeLabels, indexTypeOptions, renderHeader } from "./explorerShared"

const { activeTableId, schema, setError, setStatus } = useWorkspace()

const hasActiveTable = computed(() => Boolean(activeTableId.value))
const allFieldNames = computed(() => schema.value?.fields.map((field) => field.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((name) => ({ label: name, value: name }))
)

const distanceTypeOptions: SelectOption[] = [
	{ label: "L2", value: "l2" },
	{ label: "Cosine", value: "cosine" },
	{ label: "Dot", value: "dot" },
	{ label: "Hamming", value: "hamming" },
]

const vectorIndexTypes = new Set<IndexTypeV1>([
	"ivf_flat",
	"ivf_sq",
	"ivf_pq",
	"ivf_rq",
	"ivf_hnsw_pq",
	"ivf_hnsw_sq",
])
const pqIndexTypes = new Set<IndexTypeV1>(["ivf_pq", "ivf_hnsw_pq"])
const rqIndexTypes = new Set<IndexTypeV1>(["ivf_rq"])
const hnswIndexTypes = new Set<IndexTypeV1>(["ivf_hnsw_pq", "ivf_hnsw_sq"])

const indexes = ref<IndexDefinitionV1[]>([])
const isLoadingIndexes = ref(false)
const indexError = ref("")
const createFormOpen = ref(false)

function formatIndexStatus(row: IndexDefinitionV1) {
	if (row.numUnindexedRows === undefined || row.numIndexedRows === undefined) {
		return "未知"
	}
	if (row.numUnindexedRows === 0) {
		return `已索引 ${row.numIndexedRows} 行`
	}
	return `待索引 ${row.numUnindexedRows} 行`
}

function isPresentIndexParameter(entry: {
	label: string
	value: IndexDefinitionV1["distanceType"] | number | undefined
}) {
	return entry.value !== undefined && entry.value !== null
}

function getIndexParameterEntries(row: IndexDefinitionV1) {
	return [
		{ label: "距离", value: row.distanceType },
		{ label: "分片", value: row.numIndices },
		{ label: "Loss", value: row.loss },
	].filter(isPresentIndexParameter)
}

function renderIndexStatus(row: IndexDefinitionV1) {
	const label = formatIndexStatus(row)
	const type =
		row.numUnindexedRows === undefined
			? "default"
			: row.numUnindexedRows === 0
				? "success"
				: "warning"
	return h(NTag, { size: "small", type, bordered: false }, { default: () => label })
}

function renderIndexParameters(row: IndexDefinitionV1) {
	const entries = getIndexParameterEntries(row)
	if (!entries.length) {
		return h("span", { class: "muted-cell" }, "—")
	}
	return h(
		"div",
		{ class: "index-param-list" },
		entries.map((entry) =>
			h("span", { class: "index-param" }, [
				h("span", { class: "index-param-label" }, `${entry.label}:`),
				h("span", { class: "index-param-value" }, String(entry.value)),
			])
		)
	)
}

const indexColumns: DataTableColumns<IndexDefinitionV1> = [
	{
		title: () => renderHeader("索引名"),
		key: "name",
		width: 150,
		ellipsis: { tooltip: true },
	},
	{
		title: () => renderHeader("类型"),
		key: "indexType",
		width: 118,
		ellipsis: { tooltip: true },
		render: (row) =>
			h(
				NTag,
				{ size: "small", bordered: false },
				{ default: () => indexTypeLabels[row.indexType] ?? row.indexType }
			),
	},
	{
		title: () => renderHeader("列"),
		key: "columns",
		width: 150,
		ellipsis: { tooltip: true },
		render: (row) => row.columns.join(", "),
	},
	{
		title: () => renderHeader("状态"),
		key: "status",
		width: 140,
		ellipsis: { tooltip: true },
		render: renderIndexStatus,
	},
	{
		title: () => renderHeader("参数"),
		key: "parameters",
		width: 170,
		render: renderIndexParameters,
	},
	{
		title: () => renderHeader("操作"),
		key: "actions",
		width: 90,
		fixed: "right",
		align: "right",
		render: (row) =>
			h(
				NPopconfirm,
				{
					positiveText: "删除",
					negativeText: "取消",
					onPositiveClick: () => submitDropIndex(row.name),
				},
				{
					default: () => `确定删除索引 ${row.name} 吗？`,
					trigger: () =>
						h(
							NButton,
							{
								size: "tiny",
								type: "error",
								secondary: true,
								loading: isDroppingIndex.value && dropIndexName.value === row.name,
								disabled: !hasActiveTable.value,
							},
							{
								icon: () => h(Trash2, { class: "h-3.5 w-3.5" }),
								default: () => "删除",
							}
						),
				}
			),
	},
]

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

const indexType = ref<IndexTypeV1>("auto")
const indexColumnsToCreate = ref<string[]>([])
const indexName = ref("")
const indexReplace = ref(true)
const distanceType = ref<DistanceTypeV1 | null>(null)
const numPartitions = ref<number | null>(null)
const sampleRate = ref<number | null>(null)
const maxIterations = ref<number | null>(null)
const targetPartitionSize = ref<number | null>(null)
const numSubVectors = ref<number | null>(null)
const numBits = ref<number | null>(null)
const numEdges = ref<number | null>(null)
const efConstruction = ref<number | null>(null)
const { execute: execCreateIndex, isLoading: isCreatingIndex } = useCommand("创建索引失败")

const isVectorIndex = computed(() => vectorIndexTypes.has(indexType.value))
const isPqIndex = computed(() => pqIndexTypes.has(indexType.value))
const isRqIndex = computed(() => rqIndexTypes.has(indexType.value))
const isHnswIndex = computed(() => hnswIndexTypes.has(indexType.value))
const showsQuantizationOptions = computed(() => isPqIndex.value || isRqIndex.value)

function optionalNumber(value: number | null) {
	return value ?? undefined
}

function buildCreateIndexRequest(tableId: string, columns: string[]): CreateIndexRequestV1 {
	const request: CreateIndexRequestV1 = {
		tableId,
		columns,
		indexType: indexType.value,
		name: indexName.value.trim() || undefined,
		replace: indexReplace.value,
	}

	if (isVectorIndex.value) {
		request.distanceType = distanceType.value ?? undefined
		request.numPartitions = optionalNumber(numPartitions.value)
		request.sampleRate = optionalNumber(sampleRate.value)
		request.maxIterations = optionalNumber(maxIterations.value)
		request.targetPartitionSize = optionalNumber(targetPartitionSize.value)
	}
	if (isPqIndex.value) {
		request.numSubVectors = optionalNumber(numSubVectors.value)
		request.numBits = optionalNumber(numBits.value)
	}
	if (isRqIndex.value) {
		request.numBits = optionalNumber(numBits.value)
	}
	if (isHnswIndex.value) {
		request.numEdges = optionalNumber(numEdges.value)
		request.efConstruction = optionalNumber(efConstruction.value)
	}

	return request
}

function resetCreateForm() {
	indexName.value = ""
	indexColumnsToCreate.value = []
	distanceType.value = null
	numPartitions.value = null
	sampleRate.value = null
	maxIterations.value = null
	targetPartitionSize.value = null
	numSubVectors.value = null
	numBits.value = null
	numEdges.value = null
	efConstruction.value = null
}

async function submitCreateIndex() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}
	const columns = indexColumnsToCreate.value.map((column) => column.trim()).filter(Boolean)
	if (!columns.length) {
		setError("请选择索引列")
		return
	}
	await execCreateIndex(async () => {
		unwrapEnvelope(await createIndexV1(buildCreateIndexRequest(tableId, columns)))
		setStatus("索引创建已提交")
		await loadIndexes()
		resetCreateForm()
		createFormOpen.value = false
	})
}

const dropIndexName = ref("")
const { execute: execDropIndex, isLoading: isDroppingIndex } = useCommand("删除索引失败")

async function submitDropIndex(indexName: string) {
	const tableId = activeTableId.value
	const nameValue = indexName.trim()
	if (!tableId) {
		return
	}
	if (!nameValue) {
		setError("请选择要删除的索引")
		return
	}
	dropIndexName.value = nameValue
	try {
		await execDropIndex(async () => {
			unwrapEnvelope(await dropIndexV1(tableId, nameValue))
			setStatus(`已删除索引 ${nameValue}`)
			await loadIndexes()
		})
	} finally {
		dropIndexName.value = ""
	}
}

watch(
	activeTableId,
	() => {
		indexes.value = []
		indexError.value = ""
		indexType.value = "auto"
		indexReplace.value = true
		createFormOpen.value = false
		dropIndexName.value = ""
		resetCreateForm()
		isLoadingIndexes.value = false
		isCreatingIndex.value = false
		isDroppingIndex.value = false
		if (activeTableId.value) {
			void loadIndexes()
		}
	},
	{ immediate: true }
)
</script>

<template>
	<div class="index-workbench">
		<section class="index-table-panel">
			<header class="index-toolbar">
				<div>
					<h2 class="panel-title">索引</h2>
					<p class="panel-subtitle">
						表格视图 · {{ indexes.length }} 个索引 · {{ allFieldNames.length }} 列可选
					</p>
				</div>
				<div class="index-toolbar-actions">
					<NButton
						secondary
						size="small"
						:loading="isLoadingIndexes"
						:disabled="!hasActiveTable"
						@click="loadIndexes"
					>
						<template #icon>
							<RefreshCw class="h-4 w-4" />
						</template>
						刷新
					</NButton>
					<NButton
						type="primary"
						secondary
						size="small"
						:disabled="!hasActiveTable"
						@click="createFormOpen = !createFormOpen"
					>
						<template #icon>
							<Plus class="h-4 w-4" />
						</template>
						{{ createFormOpen ? "收起创建" : "新建索引" }}
					</NButton>
				</div>
			</header>

			<NAlert v-if="indexError" type="error" :bordered="false" class="panel-alert">
				{{ indexError }}
			</NAlert>

			<div v-show="createFormOpen" class="index-create-editor">
				<div class="create-primary-grid">
					<label class="command-field">
						<span>类型</span>
						<NSelect
							v-model:value="indexType"
							:options="indexTypeOptions"
							:disabled="!hasActiveTable"
						/>
					</label>
					<label class="command-field command-field--columns">
						<span>索引列</span>
						<NSelect
							v-model:value="indexColumnsToCreate"
							:options="columnOptions"
							multiple
							clearable
							:disabled="!hasActiveTable"
						/>
					</label>
					<label class="command-field">
						<span>名称</span>
						<NInput
							v-model:value="indexName"
							placeholder="my_index（可选）"
							:disabled="!hasActiveTable"
						/>
					</label>
					<div class="create-actions">
						<NCheckbox v-model:checked="indexReplace" :disabled="!hasActiveTable">
							替换同名
						</NCheckbox>
						<NButton
							type="primary"
							size="small"
							:loading="isCreatingIndex"
							:disabled="!hasActiveTable"
							@click="submitCreateIndex"
						>
							创建索引
						</NButton>
					</div>
				</div>

				<NAlert
					v-if="indexType === 'fts'"
					type="info"
					:bordered="false"
					class="command-note"
				>
					FTS 本轮只开放列、名称与 replace。
				</NAlert>

				<div v-if="isVectorIndex" class="tuning-section">
					<div class="tuning-heading">向量参数</div>
					<div class="tuning-grid">
						<label class="command-field">
							<span>距离</span>
							<NSelect
								v-model:value="distanceType"
								:options="distanceTypeOptions"
								clearable
								:disabled="!hasActiveTable"
							/>
						</label>
						<label class="command-field">
							<span>IVF</span>
							<NInputNumber v-model:value="numPartitions" :min="1" />
						</label>
						<label class="command-field">
							<span>Sample</span>
							<NInputNumber v-model:value="sampleRate" :min="1" />
						</label>
						<label class="command-field">
							<span>Iter</span>
							<NInputNumber v-model:value="maxIterations" :min="1" />
						</label>
						<label class="command-field">
							<span>Partition size</span>
							<NInputNumber v-model:value="targetPartitionSize" :min="1" />
						</label>
					</div>
				</div>

				<div v-if="showsQuantizationOptions" class="tuning-section">
					<div class="tuning-heading">量化</div>
					<div class="tuning-grid tuning-grid--compact">
						<label class="command-field">
							<span>Sub-vectors</span>
							<NInputNumber v-model:value="numSubVectors" :min="1" />
						</label>
						<label class="command-field">
							<span>Bits</span>
							<NInputNumber v-model:value="numBits" :min="1" />
						</label>
					</div>
				</div>

				<div v-if="isHnswIndex" class="tuning-section">
					<div class="tuning-heading">HNSW</div>
					<div class="tuning-grid tuning-grid--compact">
						<label class="command-field">
							<span>Edges</span>
							<NInputNumber v-model:value="numEdges" :min="1" />
						</label>
						<label class="command-field">
							<span>EF construction</span>
							<NInputNumber v-model:value="efConstruction" :min="1" />
						</label>
					</div>
				</div>
			</div>

			<NDataTable
				class="data-table index-table"
				size="small"
				:columns="indexColumns"
				:data="indexes"
				:loading="isLoadingIndexes"
				:bordered="false"
				:scroll-x="818"
			/>
		</section>
	</div>
</template>

<style scoped>
.index-workbench {
	min-width: 0;
}

.index-table-panel {
	min-width: 0;
	overflow: hidden;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
}

.index-toolbar {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	padding: 16px 18px 12px;
	border-bottom: 1px solid var(--app-rule);
}

.index-toolbar-actions {
	display: flex;
	flex-wrap: wrap;
	justify-content: flex-end;
	gap: 8px;
}

.panel-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 15px;
	font-weight: 680;
	line-height: 1.3;
}

.panel-subtitle {
	margin: 4px 0 0;
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.4;
}

.panel-alert {
	margin: 12px 16px 0;
}

.index-create-editor {
	padding: 14px 16px 16px;
	border-bottom: 1px solid var(--app-rule);
	background: color-mix(in srgb, var(--app-surface-panel-muted) 58%, transparent);
}

.create-primary-grid {
	display: grid;
	grid-template-columns: minmax(150px, 0.75fr) minmax(220px, 1.35fr) minmax(170px, 0.95fr) auto;
	align-items: end;
	gap: 10px;
}

.command-field {
	display: grid;
	min-width: 0;
	gap: 5px;
	color: var(--app-muted);
	font-size: 12px;
	font-weight: 620;
	line-height: 1.2;
}

.command-field--columns {
	min-width: 220px;
}

.command-field :deep(.n-input-number),
.command-field :deep(.n-input),
.command-field :deep(.n-base-selection) {
	width: 100%;
}

.command-note {
	margin-top: 10px;
}

.create-actions {
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 10px;
	min-width: 176px;
}

.tuning-section {
	display: grid;
	grid-template-columns: 96px minmax(0, 1fr);
	gap: 10px;
	margin-top: 12px;
	padding-top: 12px;
	border-top: 1px solid color-mix(in srgb, var(--app-rule) 72%, transparent);
}

.tuning-heading {
	padding-top: 6px;
	color: var(--app-ink);
	font-size: 12px;
	font-weight: 680;
}

.tuning-grid {
	display: grid;
	grid-template-columns: repeat(5, minmax(0, 1fr));
	gap: 10px;
}

.tuning-grid--compact {
	grid-template-columns: repeat(2, minmax(140px, 1fr));
	justify-content: start;
	max-width: 440px;
}

.index-table {
	padding: 0 12px 12px;
}

.muted-cell {
	color: var(--app-subtle);
}

.index-param-list {
	display: flex;
	min-width: 0;
	flex-wrap: wrap;
	gap: 4px 8px;
}

.index-param {
	display: inline-flex;
	min-width: 0;
	align-items: baseline;
	gap: 3px;
	color: var(--app-ink);
	font-family: var(--app-mono-font);
	font-size: 11px;
	line-height: 1.4;
}

.index-param-label {
	color: var(--app-muted);
	font-family: var(--app-font);
	font-size: 11px;
	font-weight: 620;
}

.index-param-value {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

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

@media (max-width: 1180px) {
	.create-primary-grid,
	.tuning-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}

	.create-actions,
	.tuning-section {
		grid-column: 1 / -1;
	}

	.tuning-section {
		grid-template-columns: 1fr;
	}

	.tuning-heading {
		padding-top: 0;
	}
}

@media (max-width: 720px) {
	.index-toolbar,
	.create-actions {
		flex-direction: column;
		align-items: stretch;
	}

	.create-primary-grid,
	.tuning-grid {
		grid-template-columns: 1fr;
	}

	.command-field--columns {
		min-width: 0;
	}
}
</style>

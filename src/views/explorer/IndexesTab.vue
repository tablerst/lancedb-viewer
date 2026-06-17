<script setup lang="ts">
import { Plus, RefreshCw, Trash2 } from "lucide-vue-next"
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, ref, watch } from "vue"

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

function formatIndexStatus(row: IndexDefinitionV1) {
	if (row.numUnindexedRows === undefined || row.numIndexedRows === undefined) {
		return "未知"
	}
	if (row.numUnindexedRows === 0) {
		return `已索引 ${row.numIndexedRows} 行`
	}
	return `待索引 ${row.numUnindexedRows} 行`
}

function formatOptional(value: unknown) {
	return value === undefined || value === null || value === "" ? "—" : String(value)
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
	{
		title: () => renderHeader("状态"),
		key: "status",
		ellipsis: { tooltip: true },
		render: formatIndexStatus,
	},
	{
		title: () => renderHeader("距离"),
		key: "distanceType",
		ellipsis: { tooltip: true },
		render: (row) => formatOptional(row.distanceType),
	},
	{
		title: () => renderHeader("分片"),
		key: "numIndices",
		ellipsis: { tooltip: true },
		render: (row) => formatOptional(row.numIndices),
	},
	{
		title: () => renderHeader("Loss"),
		key: "loss",
		ellipsis: { tooltip: true },
		render: (row) => formatOptional(row.loss),
	},
]

const indexNameOptions = computed<SelectOption[]>(() =>
	indexes.value.map((index) => ({ label: index.name, value: index.name }))
)

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
	})
}

const dropIndexName = ref("")
const { execute: execDropIndex, isLoading: isDroppingIndex } = useCommand("删除索引失败")

async function submitDropIndex() {
	const tableId = activeTableId.value
	const nameValue = dropIndexName.value.trim()
	if (!tableId) {
		return
	}
	if (!nameValue) {
		setError("请选择要删除的索引")
		return
	}
	await execDropIndex(async () => {
		unwrapEnvelope(await dropIndexV1(tableId, nameValue))
		setStatus(`已删除索引 ${nameValue}`)
		await loadIndexes()
		dropIndexName.value = ""
	})
}

watch(
	activeTableId,
	() => {
		indexes.value = []
		indexError.value = ""
		indexType.value = "auto"
		indexReplace.value = true
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
		<section class="index-main-panel">
			<header class="panel-heading">
				<div>
					<h2 class="panel-title">索引</h2>
					<p class="panel-subtitle">当前表共 {{ indexes.length }} 个索引</p>
				</div>
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
			</header>

			<NAlert v-if="indexError" type="error" :bordered="false" class="panel-alert">
				{{ indexError }}
			</NAlert>

			<NDataTable
				class="data-table index-table"
				size="small"
				:columns="indexColumns"
				:data="indexes"
				:loading="isLoadingIndexes"
				:bordered="false"
			/>
		</section>

		<aside class="index-command-panel">
			<section class="command-section">
				<div class="command-heading">
					<div>
						<h3 class="command-title">创建索引</h3>
						<p class="command-subtitle">选择列和类型，必要时再调整高级参数</p>
					</div>
					<Plus class="command-icon" />
				</div>

				<div class="command-grid">
					<label class="command-field">
						<span>索引类型</span>
						<NSelect
							v-model:value="indexType"
							:options="indexTypeOptions"
							:disabled="!hasActiveTable"
						/>
					</label>
					<label class="command-field">
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
						<span>索引名称</span>
						<NInput
							v-model:value="indexName"
							placeholder="my_index（可选）"
							:disabled="!hasActiveTable"
						/>
					</label>
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
						<label class="command-field command-field--wide">
							<span>Partition size</span>
							<NInputNumber v-model:value="targetPartitionSize" :min="1" />
						</label>
					</div>
				</div>

				<div v-if="showsQuantizationOptions" class="tuning-section">
					<div class="tuning-heading">量化</div>
					<div class="tuning-grid">
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
					<div class="tuning-grid">
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

				<div class="command-actions">
					<NCheckbox v-model:checked="indexReplace" :disabled="!hasActiveTable">
						替换同名
					</NCheckbox>
					<NButton
						type="primary"
						:loading="isCreatingIndex"
						:disabled="!hasActiveTable"
						@click="submitCreateIndex"
					>
						创建
					</NButton>
				</div>
			</section>

			<section class="command-section command-section--danger">
				<div class="command-heading">
					<div>
						<h3 class="command-title">维护</h3>
						<p class="command-subtitle">删除不再需要的索引</p>
					</div>
					<Trash2 class="command-icon command-icon--danger" />
				</div>
				<label class="command-field">
					<span>索引</span>
					<NSelect
						v-model:value="dropIndexName"
						:options="indexNameOptions"
						clearable
						:disabled="!hasActiveTable || !indexes.length"
					/>
				</label>
				<div class="command-actions command-actions--end">
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
								:disabled="!hasActiveTable || !indexes.length"
							>
								删除索引
							</NButton>
						</template>
						确定删除选中的索引吗？
					</NPopconfirm>
				</div>
			</section>
		</aside>
	</div>
</template>

<style scoped>
.index-workbench {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(320px, 380px);
	gap: 16px;
	align-items: start;
}

.index-main-panel,
.index-command-panel {
	min-width: 0;
}

.index-main-panel,
.command-section {
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
}

.index-main-panel {
	overflow: hidden;
}

.panel-heading {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	padding: 16px 18px 12px;
	border-bottom: 1px solid var(--app-rule);
}

.panel-title,
.command-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 15px;
	font-weight: 680;
	line-height: 1.3;
}

.panel-subtitle,
.command-subtitle {
	margin: 4px 0 0;
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.4;
}

.panel-alert {
	margin: 12px 16px 0;
}

.index-table {
	padding: 0 12px 12px;
}

.index-command-panel {
	display: grid;
	gap: 12px;
}

.command-section {
	padding: 14px;
}

.command-section--danger {
	background: color-mix(in srgb, var(--app-danger-soft) 40%, var(--app-surface-elevated));
}

.command-heading {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	margin-bottom: 14px;
}

.command-icon {
	width: 18px;
	height: 18px;
	color: var(--app-subtle);
}

.command-icon--danger {
	color: var(--app-danger);
}

.command-grid,
.tuning-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 10px;
}

.command-grid {
	grid-template-columns: 1fr;
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

.command-field--wide {
	grid-column: 1 / -1;
}

.command-field :deep(.n-input-number),
.command-field :deep(.n-input),
.command-field :deep(.n-base-selection) {
	width: 100%;
}

.command-note {
	margin-top: 10px;
}

.tuning-section {
	margin-top: 14px;
	padding-top: 12px;
	border-top: 1px solid var(--app-rule);
}

.tuning-heading {
	margin-bottom: 10px;
	color: var(--app-ink);
	font-size: 12px;
	font-weight: 680;
}

.command-actions {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 10px;
	margin-top: 14px;
}

.command-actions--end {
	justify-content: flex-end;
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

@media (max-width: 1100px) {
	.index-workbench {
		grid-template-columns: 1fr;
	}

	.index-command-panel {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}

@media (max-width: 720px) {
	.panel-heading,
	.command-actions {
		flex-direction: column;
		align-items: stretch;
	}

	.index-command-panel,
	.command-grid,
	.tuning-grid {
		grid-template-columns: 1fr;
	}
}
</style>

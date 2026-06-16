<script setup lang="ts">
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
					<label class="text-sm font-medium text-slate-600">索引类型</label>
					<NSelect
						v-model:value="indexType"
						:options="indexTypeOptions"
						:disabled="!hasActiveTable"
					/>
				</div>
				<div class="xl:col-span-4">
					<label class="text-sm font-medium text-slate-600">索引列</label>
					<NSelect
						v-model:value="indexColumnsToCreate"
						:options="columnOptions"
						multiple
						clearable
						:disabled="!hasActiveTable"
					/>
				</div>
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">索引名称（可选）</label>
					<NInput
						v-model:value="indexName"
						placeholder="my_index"
						:disabled="!hasActiveTable"
					/>
				</div>
				<div class="xl:col-span-3 flex items-end justify-end gap-3">
					<NCheckbox v-model:checked="indexReplace" :disabled="!hasActiveTable">
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

				<template v-if="isVectorIndex">
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">距离类型</label>
						<NSelect
							v-model:value="distanceType"
							:options="distanceTypeOptions"
							clearable
							:disabled="!hasActiveTable"
						/>
					</div>
					<div class="xl:col-span-1">
						<label class="text-sm font-medium text-slate-600">IVF partitions</label>
						<NInputNumber v-model:value="numPartitions" :min="1" />
					</div>
					<div class="xl:col-span-1">
						<label class="text-sm font-medium text-slate-600">Sample rate</label>
						<NInputNumber v-model:value="sampleRate" :min="1" />
					</div>
					<div class="xl:col-span-1">
						<label class="text-sm font-medium text-slate-600">Max iterations</label>
						<NInputNumber v-model:value="maxIterations" :min="1" />
					</div>
					<div class="xl:col-span-1">
						<label class="text-sm font-medium text-slate-600">Partition size</label>
						<NInputNumber v-model:value="targetPartitionSize" :min="1" />
					</div>
				</template>

				<template v-if="showsQuantizationOptions">
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">Sub-vectors</label>
						<NInputNumber v-model:value="numSubVectors" :min="1" />
					</div>
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">Bits</label>
						<NInputNumber v-model:value="numBits" :min="1" />
					</div>
				</template>

				<template v-if="isHnswIndex">
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">HNSW edges</label>
						<NInputNumber v-model:value="numEdges" :min="1" />
					</div>
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">EF construction</label>
						<NInputNumber v-model:value="efConstruction" :min="1" />
					</div>
				</template>
			</div>
			<NAlert v-if="indexType === 'fts'" type="info" :bordered="false" class="mt-3">
				FTS 索引当前暴露列、名称与 replace；tokenizer 等底层参数本轮不开放。
			</NAlert>
		</NCard>

		<NCard size="small" title="删除索引" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-4">
					<label class="text-sm font-medium text-slate-600">选择索引</label>
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

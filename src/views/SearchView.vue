<script setup lang="ts">
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, h, ref, watch } from "vue"

import { useWorkspace } from "../composables/workspaceContext"
import type { SchemaDefinition } from "../ipc/v1"
import { formatCellValue, normalizeRow } from "../lib/formatters"
import {
	combinedSearchV1,
	ftsSearchV1,
	queryFilterV1,
	unwrapEnvelope,
	vectorSearchV1,
} from "../lib/tauriClient"

const {
	activeTableName,
	activeTableId,
	schema,
	setStatus,
	setError,
	clearMessages,
} = useWorkspace()

const tableHint = computed(() => activeTableName.value ?? "尚未选择表")
const hasActiveTable = computed(() => Boolean(activeTableId.value))

const allFieldNames = computed(() => schema.value?.fields.map((field) => field.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((name) => ({ label: name, value: name }))
)

const activeTab = ref("filter")

const filterExpression = ref("")
const filterLimit = ref(50)
const filterOffset = ref(0)
const filterProjection = ref<string[]>([])

const vectorText = ref("")
const vectorColumn = ref<string | null>(null)
const vectorTopK = ref(10)
const vectorOffset = ref(0)
const vectorProjection = ref<string[]>([])
const vectorFilter = ref("")
const vectorNprobes = ref<number | null>(null)
const vectorRefine = ref<number | null>(null)

const ftsQuery = ref("")
const ftsColumns = ref<string[]>([])
const ftsLimit = ref(50)
const ftsOffset = ref(0)
const ftsProjection = ref<string[]>([])
const ftsFilter = ref("")

const combinedQuery = ref("")
const combinedVectorText = ref("")
const combinedVectorColumn = ref<string | null>(null)
const combinedColumns = ref<string[]>([])
const combinedLimit = ref(50)
const combinedOffset = ref(0)
const combinedProjection = ref<string[]>([])
const combinedFilter = ref("")
const combinedNprobes = ref<number | null>(null)
const combinedRefine = ref<number | null>(null)

const isSearching = ref(false)
const resultRows = ref<unknown[]>([])
const resultSchema = ref<SchemaDefinition | null>(null)
const resultNextOffset = ref<number | null>(null)
const resultError = ref("")

function renderHeader(title: string) {
	return h("span", { class: "table-header-ellipsis", title }, title)
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

const resultColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
	const fields = resultSchema.value?.fields ?? schema.value?.fields ?? []
	return fields.map((field) => ({
		title: () => renderHeader(field.name),
		key: field.name,
		ellipsis: { tooltip: true },
		sorter: (rowA, rowB) => compareValues(rowA[field.name], rowB[field.name]),
		render: (row) => formatCellValue(row[field.name]),
	}))
})

const resultTableData = computed(() =>
	resultRows.value.map((row, index) => ({
		__rowId: `${index}`,
		...normalizeRow(row),
	}))
)

function resetResults() {
	resultRows.value = []
	resultSchema.value = null
	resultNextOffset.value = null
	resultError.value = ""
}

watch(activeTableId, () => {
	resetResults()
})

watch(activeTab, () => {
	resetResults()
})

function parseVectorInputValue(value: string) {
	const cleaned = value.replace(/[[\]]/g, " ")
	const parts = cleaned.split(/[,\s]+/).filter(Boolean)
	const numbers = parts.map((part) => Number(part))
	if (!numbers.length || numbers.some((value) => Number.isNaN(value))) {
		return null
	}
	return numbers
}

function parseVectorInput() {
	return parseVectorInputValue(vectorText.value)
}

function parseCombinedVectorInput() {
	return parseVectorInputValue(combinedVectorText.value)
}

async function runFilterQuery() {
	const tableId = activeTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(
			await queryFilterV1({
				tableId,
				filter: filterExpression.value,
				projection: filterProjection.value.length ? filterProjection.value : undefined,
				limit: filterLimit.value,
				offset: filterOffset.value,
			})
		)
		if (response.chunk.format !== "json") {
			resultError.value = "当前仅支持 JSON 数据块"
			return
		}
		resultRows.value = response.chunk.rows
		resultSchema.value = response.chunk.schema
		resultNextOffset.value = response.nextOffset ?? null
		setStatus(`已返回 ${response.chunk.rows.length} 行`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "过滤查询失败"
		resultError.value = message
		setError(message)
	} finally {
		isSearching.value = false
	}
}

async function runVectorQuery() {
	const tableId = activeTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	const vector = parseVectorInput()
	if (!vector) {
		resultError.value = "请输入有效向量（例如：0.1, 0.2, 0.3）"
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(
			await vectorSearchV1({
				tableId,
				vector,
				column: vectorColumn.value ?? undefined,
				topK: vectorTopK.value,
				offset: vectorOffset.value,
				projection: vectorProjection.value.length ? vectorProjection.value : undefined,
				filter: vectorFilter.value.trim() || undefined,
				nprobes: vectorNprobes.value ?? undefined,
				refineFactor: vectorRefine.value ?? undefined,
			})
		)
		if (response.chunk.format !== "json") {
			resultError.value = "当前仅支持 JSON 数据块"
			return
		}
		resultRows.value = response.chunk.rows
		resultSchema.value = response.chunk.schema
		resultNextOffset.value = response.nextOffset ?? null
		setStatus(`已返回 ${response.chunk.rows.length} 行`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "向量检索失败"
		resultError.value = message
		setError(message)
	} finally {
		isSearching.value = false
	}
}

async function runFtsQuery() {
	const tableId = activeTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(
			await ftsSearchV1({
				tableId,
				query: ftsQuery.value,
				columns: ftsColumns.value.length ? ftsColumns.value : undefined,
				limit: ftsLimit.value,
				offset: ftsOffset.value,
				projection: ftsProjection.value.length ? ftsProjection.value : undefined,
				filter: ftsFilter.value.trim() || undefined,
			})
		)
		if (response.chunk.format !== "json") {
			resultError.value = "当前仅支持 JSON 数据块"
			return
		}
		resultRows.value = response.chunk.rows
		resultSchema.value = response.chunk.schema
		resultNextOffset.value = response.nextOffset ?? null
		setStatus(`已返回 ${response.chunk.rows.length} 行`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "全文检索失败"
		resultError.value = message
		setError(message)
	} finally {
		isSearching.value = false
	}
}

async function runCombinedQuery() {
	const tableId = activeTableId.value
	if (!tableId || isSearching.value) {
		return
	}

	const queryText = combinedQuery.value.trim()
	const vectorInput = combinedVectorText.value.trim()
	const vector = vectorInput ? parseCombinedVectorInput() : null
	if (!queryText && !vectorInput) {
		resultError.value = "请输入向量或查询文本"
		return
	}
	if (vectorInput && !vector) {
		resultError.value = "请输入有效向量（例如：0.1, 0.2, 0.3）"
		return
	}

	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(
			await combinedSearchV1({
				tableId,
				vector: vector ?? undefined,
				vectorColumn: combinedVectorColumn.value ?? undefined,
				query: queryText || undefined,
				columns: combinedColumns.value.length ? combinedColumns.value : undefined,
				projection: combinedProjection.value.length ? combinedProjection.value : undefined,
				filter: combinedFilter.value.trim() || undefined,
				limit: combinedLimit.value,
				offset: combinedOffset.value,
				nprobes: combinedNprobes.value ?? undefined,
				refineFactor: combinedRefine.value ?? undefined,
			})
		)
		if (response.chunk.format !== "json") {
			resultError.value = "当前仅支持 JSON 数据块"
			return
		}
		resultRows.value = response.chunk.rows
		resultSchema.value = response.chunk.schema
		resultNextOffset.value = response.nextOffset ?? null
		setStatus(`已返回 ${response.chunk.rows.length} 行`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "组合查询失败"
		resultError.value = message
		setError(message)
	} finally {
		isSearching.value = false
	}
}
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="检索工作台" class="shadow-sm">
			<div class="text-xs text-slate-500">
				当前表：{{ tableHint }}。请先在“资源浏览”中选择表。
			</div>
		</NCard>

		<NEmpty v-if="!hasActiveTable" description="请先选择表" />

		<div v-else class="space-y-4">
			<NTabs v-model:value="activeTab" type="line">
				<NTabPane name="filter" tab="过滤查询">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-xs text-slate-500">过滤表达式</label>
							<NInput v-model:value="filterExpression" placeholder="id > 10" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-xs text-slate-500">Limit</label>
								<NInputNumber v-model:value="filterLimit" :min="1" />
							</div>
							<div>
								<label class="text-xs text-slate-500">Offset</label>
								<NInputNumber v-model:value="filterOffset" :min="0" />
							</div>
						</div>
					</div>
					<div class="mt-3">
						<label class="text-xs text-slate-500">列投影</label>
						<NSelect
							v-model:value="filterProjection"
							:options="columnOptions"
							multiple
							clearable
						/>
					</div>
					<div class="mt-3">
						<NButton type="primary" :loading="isSearching" @click="runFilterQuery">
							查询
						</NButton>
					</div>
				</NTabPane>

				<NTabPane name="combined" tab="组合查询">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-xs text-slate-500">全文查询（可选）</label>
							<NInput v-model:value="combinedQuery" placeholder="item 1" />
						</div>
						<div class="xl:col-span-3">
							<label class="text-xs text-slate-500">向量输入（可选）</label>
							<NInput v-model:value="combinedVectorText" placeholder="0.1, 0.2, 0.3" />
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-2">
							<label class="text-xs text-slate-500">向量列</label>
							<NSelect
								v-model:value="combinedVectorColumn"
								:options="columnOptions"
								clearable
							/>
						</div>
						<div class="xl:col-span-2">
							<label class="text-xs text-slate-500">全文列</label>
							<NSelect
								v-model:value="combinedColumns"
								:options="columnOptions"
								multiple
								clearable
							/>
						</div>
						<div class="xl:col-span-2">
							<label class="text-xs text-slate-500">Filter</label>
							<NInput v-model:value="combinedFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-2">
							<label class="text-xs text-slate-500">Limit</label>
							<NInputNumber v-model:value="combinedLimit" :min="1" />
						</div>
						<div class="xl:col-span-2">
							<label class="text-xs text-slate-500">Offset</label>
							<NInputNumber v-model:value="combinedOffset" :min="0" />
						</div>
						<div class="xl:col-span-1">
							<label class="text-xs text-slate-500">nprobes</label>
							<NInputNumber v-model:value="combinedNprobes" :min="1" />
						</div>
						<div class="xl:col-span-1">
							<label class="text-xs text-slate-500">refine</label>
							<NInputNumber v-model:value="combinedRefine" :min="1" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-xs text-slate-500">列投影</label>
						<NSelect
							v-model:value="combinedProjection"
							:options="columnOptions"
							multiple
							clearable
						/>
					</div>
					<div class="mt-3">
						<NButton type="primary" :loading="isSearching" @click="runCombinedQuery">
							组合查询
						</NButton>
					</div>
				</NTabPane>

				<NTabPane name="vector" tab="向量检索">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-xs text-slate-500">向量输入</label>
							<NInput v-model:value="vectorText" placeholder="0.1, 0.2, 0.3" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-xs text-slate-500">向量列</label>
								<NSelect
									v-model:value="vectorColumn"
									:options="columnOptions"
									clearable
								/>
							</div>
							<div>
								<label class="text-xs text-slate-500">Top K</label>
								<NInputNumber v-model:value="vectorTopK" :min="1" />
							</div>
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-4">
						<div>
							<label class="text-xs text-slate-500">Offset</label>
							<NInputNumber v-model:value="vectorOffset" :min="0" />
						</div>
						<div>
							<label class="text-xs text-slate-500">nprobes</label>
							<NInputNumber v-model:value="vectorNprobes" :min="1" />
						</div>
						<div>
							<label class="text-xs text-slate-500">refine factor</label>
							<NInputNumber v-model:value="vectorRefine" :min="1" />
						</div>
						<div>
							<label class="text-xs text-slate-500">Filter</label>
							<NInput v-model:value="vectorFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-xs text-slate-500">列投影</label>
						<NSelect
							v-model:value="vectorProjection"
							:options="columnOptions"
							multiple
							clearable
						/>
					</div>
					<div class="mt-3">
						<NButton type="primary" :loading="isSearching" @click="runVectorQuery">
							检索
						</NButton>
					</div>
				</NTabPane>

				<NTabPane name="fts" tab="全文检索">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-xs text-slate-500">查询文本</label>
							<NInput v-model:value="ftsQuery" placeholder="item 1" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-xs text-slate-500">Limit</label>
								<NInputNumber v-model:value="ftsLimit" :min="1" />
							</div>
							<div>
								<label class="text-xs text-slate-500">Offset</label>
								<NInputNumber v-model:value="ftsOffset" :min="0" />
							</div>
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-2">
						<div>
							<label class="text-xs text-slate-500">索引列</label>
							<NSelect
								v-model:value="ftsColumns"
								:options="columnOptions"
								multiple
								clearable
							/>
						</div>
						<div>
							<label class="text-xs text-slate-500">Filter</label>
							<NInput v-model:value="ftsFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-xs text-slate-500">列投影</label>
						<NSelect
							v-model:value="ftsProjection"
							:options="columnOptions"
							multiple
							clearable
						/>
					</div>
					<div class="mt-3">
						<NButton type="primary" :loading="isSearching" @click="runFtsQuery">
							检索
						</NButton>
					</div>
				</NTabPane>
			</NTabs>

			<NAlert v-if="resultError" type="error" :bordered="false">
				{{ resultError }}
			</NAlert>

			<NCard size="small" title="结果" class="shadow-sm">
				<div class="mb-2 flex items-center justify-between text-xs text-slate-500">
					<span>返回行数：{{ resultRows.length }}</span>
					<span v-if="resultNextOffset !== null">nextOffset: {{ resultNextOffset }}</span>
				</div>
				<NDataTable
					class="data-table"
					size="small"
					:columns="resultColumns"
					:data="resultTableData"
					:bordered="false"
					:loading="isSearching"
					:row-key="(row) => row.__rowId"
				/>
			</NCard>
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
</style>

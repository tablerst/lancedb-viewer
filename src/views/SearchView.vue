<script setup lang="ts">
import { Database, Search } from "lucide-vue-next"
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue"
import { useRoute } from "vue-router"

import DataResultTable from "../components/DataResultTable.vue"
import { useWorkspace } from "../composables/workspaceContext"
import type { SchemaDefinition } from "../ipc/v1"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "../lib/connectionKind"
import { renderCellValue } from "../lib/formatters"
import {
	combinedSearchV1,
	ftsSearchV1,
	queryFilterV1,
	unwrapEnvelope,
	vectorSearchV1,
} from "../lib/tauriClient"
import { compareValues, renderHeader } from "./explorer/explorerShared"
import {
	buildCombinedSearchRequest,
	buildFilterQueryRequest,
	buildFtsSearchRequest,
	buildVectorSearchRequest,
} from "./search/searchRequests"

const {
	profiles,
	activeProfileId,
	connectionStates,
	openTable,
	setStatus,
	setError,
	clearMessages,
} = useWorkspace()

const route = useRoute()

const routeProfileId = computed(() => {
	const raw = route.params.id
	return typeof raw === "string" ? raw : null
})

const scopedProfileId = computed(() => routeProfileId.value ?? activeProfileId.value)
const scopedProfile = computed(() => {
	const id = scopedProfileId.value
	if (!id) {
		return null
	}
	return profiles.value.find((item) => item.id === id) ?? null
})

const scopedConnection = computed(() => {
	const id = scopedProfileId.value
	if (!id) {
		return null
	}
	return connectionStates.value[id] ?? null
})

const scopedConnectionId = computed(() => scopedConnection.value?.connectionId.value ?? null)
const scopedTables = computed(() => scopedConnection.value?.tables.value ?? [])
const scopedActiveTableName = computed(() => scopedConnection.value?.activeTableName.value ?? null)
const scopedActiveTableId = computed(() => scopedConnection.value?.activeTableId.value ?? null)
const scopedSchema = computed(() => scopedConnection.value?.schema.value ?? null)

const connectionKindLabel = computed(() => {
	const profile = scopedProfile.value
	if (!profile) {
		return null
	}
	return getConnectionKindLabel(getConnectionKind(profile.uri))
})

const connectionKindTagType = computed(() => {
	const profile = scopedProfile.value
	if (!profile) {
		return "default"
	}
	return getConnectionKindTagType(getConnectionKind(profile.uri))
})

const hasActiveTable = computed(() => Boolean(scopedActiveTableId.value))

const tableOptions = computed<SelectOption[]>(() =>
	scopedTables.value.map((item) => ({ label: item.name, value: item.name }))
)

async function onTableSelect(name: string | null) {
	const id = scopedProfileId.value
	if (!name || !id || !scopedConnectionId.value) {
		return
	}
	clearMessages()
	await openTable(id, name)
}

const allFieldNames = computed(() => scopedSchema.value?.fields.map((field) => field.name) ?? [])
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

const searchMetadataFields = [
	"_relevance_score",
	"_distance",
	"_score",
	"_hybrid_rank",
	"_hybrid_source",
]

const searchMetadataLabels: Record<string, string> = {
	_relevance_score: "融合分数",
	_distance: "向量距离",
	_score: "全文分数",
	_hybrid_rank: "融合排序",
	_hybrid_source: "来源",
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return Boolean(value) && typeof value === "object" && !Array.isArray(value)
}

const resultColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
	const fields = resultSchema.value?.fields ?? scopedSchema.value?.fields ?? []
	const fieldNames = new Set(fields.map((field) => field.name))
	const firstRow = resultRows.value.find(isRecord)
	const metadataFields = searchMetadataFields
		.filter((name) => !fieldNames.has(name) && firstRow?.[name] !== undefined)
		.map((name) => ({ name, dataType: "metadata", nullable: true, metadata: undefined }))
	return [...fields, ...metadataFields].map((field) => ({
		title: () => renderHeader(field.name),
		key: field.name,
		ellipsis: { tooltip: true },
		sorter: (rowA, rowB) => compareValues(rowA[field.name], rowB[field.name]),
		render: (row) => renderCellValue(row[field.name]),
	}))
})

const scoreSummaries = computed(() => {
	const firstRow = resultRows.value.find(isRecord)
	return searchMetadataFields.map((name) => ({
		name,
		label: searchMetadataLabels[name],
		value: firstRow?.[name],
	}))
})

const canGoPrevious = computed(() => {
	switch (activeTab.value) {
		case "filter":
			return filterOffset.value > 0
		case "combined":
			return combinedOffset.value > 0
		case "vector":
			return vectorOffset.value > 0
		case "fts":
			return ftsOffset.value > 0
		default:
			return false
	}
})

function getActiveLimit() {
	switch (activeTab.value) {
		case "filter":
			return filterLimit.value
		case "combined":
			return combinedLimit.value
		case "vector":
			return vectorTopK.value
		case "fts":
			return ftsLimit.value
		default:
			return 50
	}
}

function setActiveOffset(offset: number) {
	const nextOffset = Math.max(0, offset)
	switch (activeTab.value) {
		case "filter":
			filterOffset.value = nextOffset
			break
		case "combined":
			combinedOffset.value = nextOffset
			break
		case "vector":
			vectorOffset.value = nextOffset
			break
		case "fts":
			ftsOffset.value = nextOffset
			break
	}
}

function resetResults() {
	resultRows.value = []
	resultSchema.value = null
	resultNextOffset.value = null
	resultError.value = ""
}

watch(scopedActiveTableId, () => {
	resetResults()
})

watch(scopedProfileId, () => {
	resetResults()
})

watch(activeTab, () => {
	resetResults()
})

// ── Keyboard shortcuts ─────────────────────────────────

function runActiveQuery() {
	if (!hasActiveTable.value || isSearching.value) {
		return
	}
	switch (activeTab.value) {
		case "filter":
			void runFilterQuery()
			break
		case "combined":
			void runCombinedQuery()
			break
		case "vector":
			void runVectorQuery()
			break
		case "fts":
			void runFtsQuery()
			break
	}
}

function goPreviousPage() {
	if (!canGoPrevious.value || isSearching.value) {
		return
	}
	const limit = getActiveLimit()
	const currentOffset = (() => {
		switch (activeTab.value) {
			case "filter":
				return filterOffset.value
			case "combined":
				return combinedOffset.value
			case "vector":
				return vectorOffset.value
			case "fts":
				return ftsOffset.value
			default:
				return 0
		}
	})()
	setActiveOffset(currentOffset - limit)
	runActiveQuery()
}

function goNextPage() {
	if (resultNextOffset.value === null || isSearching.value) {
		return
	}
	setActiveOffset(resultNextOffset.value)
	runActiveQuery()
}

function handleKeydown(event: KeyboardEvent) {
	if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
		event.preventDefault()
		runActiveQuery()
	}
}

onMounted(() => window.addEventListener("keydown", handleKeydown))
onBeforeUnmount(() => window.removeEventListener("keydown", handleKeydown))

async function runFilterQuery() {
	const tableId = scopedActiveTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	const candidate = buildFilterQueryRequest({
		tableId,
		filter: filterExpression.value,
		projection: filterProjection.value,
		limit: filterLimit.value,
		offset: filterOffset.value,
	})
	if (!candidate.ok) {
		resultError.value = candidate.message
		clearMessages()
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(await queryFilterV1(candidate.request))
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
	const tableId = scopedActiveTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	const candidate = buildVectorSearchRequest({
		tableId,
		vectorText: vectorText.value,
		column: vectorColumn.value,
		topK: vectorTopK.value,
		offset: vectorOffset.value,
		projection: vectorProjection.value,
		filter: vectorFilter.value,
		nprobes: vectorNprobes.value,
		refineFactor: vectorRefine.value,
	})
	if (!candidate.ok) {
		resultError.value = candidate.message
		clearMessages()
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(await vectorSearchV1(candidate.request))
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
	const tableId = scopedActiveTableId.value
	if (!tableId || isSearching.value) {
		return
	}
	const candidate = buildFtsSearchRequest({
		tableId,
		query: ftsQuery.value,
		columns: ftsColumns.value,
		limit: ftsLimit.value,
		offset: ftsOffset.value,
		projection: ftsProjection.value,
		filter: ftsFilter.value,
	})
	if (!candidate.ok) {
		resultError.value = candidate.message
		clearMessages()
		return
	}
	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(await ftsSearchV1(candidate.request))
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
	const tableId = scopedActiveTableId.value
	if (!tableId || isSearching.value) {
		return
	}

	const candidate = buildCombinedSearchRequest({
		tableId,
		query: combinedQuery.value,
		vectorText: combinedVectorText.value,
		vectorColumn: combinedVectorColumn.value,
		columns: combinedColumns.value,
		limit: combinedLimit.value,
		offset: combinedOffset.value,
		projection: combinedProjection.value,
		filter: combinedFilter.value,
		nprobes: combinedNprobes.value,
		refineFactor: combinedRefine.value,
	})
	if (!candidate.ok) {
		resultError.value = candidate.message
		clearMessages()
		return
	}

	try {
		isSearching.value = true
		resultError.value = ""
		clearMessages()
		const response = unwrapEnvelope(await combinedSearchV1(candidate.request))
		if (response.chunk.format !== "json") {
			resultError.value = "当前仅支持 JSON 数据块"
			return
		}
		resultRows.value = response.chunk.rows
		resultSchema.value = response.chunk.schema
		resultNextOffset.value = response.nextOffset ?? null
		setStatus(`已返回 ${response.chunk.rows.length} 行`)
	} catch (error) {
		const message = error instanceof Error ? error.message : "混合检索失败"
		resultError.value = message
		setError(message)
	} finally {
		isSearching.value = false
	}
}
</script>

<template>
	<div class="space-y-4">
		<div
			v-if="!scopedProfileId"
			class="flex flex-col items-center justify-center gap-3 py-20 text-center"
		>
			<div
				class="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="text-base font-semibold text-slate-700">选择连接</div>
				<div class="mt-1 text-sm text-slate-500">
					请先在侧栏选择连接，再进行检索
				</div>
			</div>
		</div>
		<div
			v-else-if="!scopedConnectionId"
			class="flex flex-col items-center justify-center gap-3 py-20 text-center"
		>
			<div
				class="flex h-14 w-14 items-center justify-center rounded-2xl bg-amber-50 text-amber-400"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="text-base font-semibold text-slate-700">尚未连接</div>
				<div class="mt-1 text-sm text-slate-500">
					当前连接尚未建立，请在侧栏中点击连接
				</div>
			</div>
		</div>
		<div
			v-else-if="!hasActiveTable"
			class="flex flex-col items-center justify-center gap-3 py-20 text-center"
		>
			<div
				class="flex h-14 w-14 items-center justify-center rounded-2xl bg-sky-50 text-sky-400"
			>
				<Search class="h-7 w-7" />
			</div>
			<div>
				<div class="text-base font-semibold text-slate-700">选择要检索的表</div>
				<div class="mt-1 text-sm text-slate-500">
					从侧栏中选择表，或使用下方快速切换
				</div>
			</div>
			<NSelect
				v-if="scopedTables.length"
				:value="scopedActiveTableName"
				:options="tableOptions"
				filterable
				clearable
				placeholder="快速选择表"
				size="small"
				class="mt-2 w-60"
				@update:value="onTableSelect"
			/>
		</div>

		<div v-else class="space-y-4">
			<div
				class="flex items-center justify-between rounded-lg border border-slate-100 bg-white px-4 py-2 text-sm"
			>
				<div class="flex items-center gap-2 text-slate-600">
					<span>当前表：</span>
					<span class="font-medium text-slate-800">{{
						scopedActiveTableName
					}}</span>
					<NTag size="small" :type="connectionKindTagType">
						{{ scopedProfile?.name }} · {{ connectionKindLabel }}
					</NTag>
				</div>
				<NSelect
					:value="scopedActiveTableName"
					:options="tableOptions"
					filterable
					size="small"
					class="w-48"
					@update:value="onTableSelect"
				/>
			</div>

			<NTabs v-model:value="activeTab" type="line">
				<NTabPane name="filter" tab="过滤查询">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-sm font-medium text-slate-600">过滤表达式</label>
							<NInput v-model:value="filterExpression" placeholder="id > 10" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-sm font-medium text-slate-600">Limit</label>
								<NInputNumber v-model:value="filterLimit" :min="1" />
							</div>
							<div>
								<label class="text-sm font-medium text-slate-600">Offset</label>
								<NInputNumber v-model:value="filterOffset" :min="0" />
							</div>
						</div>
					</div>
					<div class="mt-3">
						<label class="text-sm font-medium text-slate-600">列投影</label>
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

				<NTabPane name="combined" tab="混合检索">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-sm font-medium text-slate-600">全文查询</label>
							<NInput v-model:value="combinedQuery" placeholder="item 1" />
						</div>
						<div class="xl:col-span-3">
							<label class="text-sm font-medium text-slate-600">向量输入</label>
							<NInput v-model:value="combinedVectorText" placeholder="0.1, 0.2, 0.3" />
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-2">
							<label class="text-sm font-medium text-slate-600">向量列</label>
							<NSelect
								v-model:value="combinedVectorColumn"
								:options="columnOptions"
								clearable
							/>
						</div>
						<div class="xl:col-span-2">
							<label class="text-sm font-medium text-slate-600">全文列</label>
							<NSelect
								v-model:value="combinedColumns"
								:options="columnOptions"
								multiple
								clearable
							/>
						</div>
						<div class="xl:col-span-2">
							<label class="text-sm font-medium text-slate-600">Filter</label>
							<NInput v-model:value="combinedFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-2">
							<label class="text-sm font-medium text-slate-600">Limit</label>
							<NInputNumber v-model:value="combinedLimit" :min="1" />
						</div>
						<div class="xl:col-span-2">
							<label class="text-sm font-medium text-slate-600">Offset</label>
							<NInputNumber v-model:value="combinedOffset" :min="0" />
						</div>
						<div class="xl:col-span-1">
							<label class="text-sm font-medium text-slate-600">nprobes</label>
							<NInputNumber v-model:value="combinedNprobes" :min="1" />
						</div>
						<div class="xl:col-span-1">
							<label class="text-sm font-medium text-slate-600">refine</label>
							<NInputNumber v-model:value="combinedRefine" :min="1" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-sm font-medium text-slate-600">列投影</label>
						<NSelect
							v-model:value="combinedProjection"
							:options="columnOptions"
							multiple
							clearable
						/>
					</div>
					<div class="mt-3">
						<NButton type="primary" :loading="isSearching" @click="runCombinedQuery">
							混合检索
						</NButton>
					</div>
				</NTabPane>

				<NTabPane name="vector" tab="向量检索">
					<div class="grid gap-3 xl:grid-cols-6">
						<div class="xl:col-span-3">
							<label class="text-sm font-medium text-slate-600">向量输入</label>
							<NInput v-model:value="vectorText" placeholder="0.1, 0.2, 0.3" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-sm font-medium text-slate-600">向量列</label>
								<NSelect
									v-model:value="vectorColumn"
									:options="columnOptions"
									clearable
								/>
							</div>
							<div>
								<label class="text-sm font-medium text-slate-600">Top K</label>
								<NInputNumber v-model:value="vectorTopK" :min="1" />
							</div>
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-4">
						<div>
							<label class="text-sm font-medium text-slate-600">Offset</label>
							<NInputNumber v-model:value="vectorOffset" :min="0" />
						</div>
						<div>
							<label class="text-sm font-medium text-slate-600">nprobes</label>
							<NInputNumber v-model:value="vectorNprobes" :min="1" />
						</div>
						<div>
							<label class="text-sm font-medium text-slate-600">refine factor</label>
							<NInputNumber v-model:value="vectorRefine" :min="1" />
						</div>
						<div>
							<label class="text-sm font-medium text-slate-600">Filter</label>
							<NInput v-model:value="vectorFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-sm font-medium text-slate-600">列投影</label>
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
							<label class="text-sm font-medium text-slate-600">查询文本</label>
							<NInput v-model:value="ftsQuery" placeholder="item 1" />
						</div>
						<div class="xl:col-span-3 grid grid-cols-2 gap-3">
							<div>
								<label class="text-sm font-medium text-slate-600">Limit</label>
								<NInputNumber v-model:value="ftsLimit" :min="1" />
							</div>
							<div>
								<label class="text-sm font-medium text-slate-600">Offset</label>
								<NInputNumber v-model:value="ftsOffset" :min="0" />
							</div>
						</div>
					</div>
					<div class="mt-3 grid gap-3 xl:grid-cols-2">
						<div>
							<label class="text-sm font-medium text-slate-600">索引列</label>
							<NSelect
								v-model:value="ftsColumns"
								:options="columnOptions"
								multiple
								clearable
							/>
						</div>
						<div>
							<label class="text-sm font-medium text-slate-600">Filter</label>
							<NInput v-model:value="ftsFilter" placeholder="id > 10" />
						</div>
					</div>
					<div class="mt-3">
						<label class="text-sm font-medium text-slate-600">列投影</label>
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

			<NCard size="small" title="结果" class="bg-slate-50/60 shadow-sm">
				<div class="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs text-slate-500">
					<div class="flex flex-wrap items-center gap-2">
						<span>返回行数：{{ resultRows.length }}</span>
						<NTag
							v-for="item in scoreSummaries"
							:key="item.name"
							size="small"
							:bordered="false"
						>
							{{ item.label }}:
							{{ item.value === undefined || item.value === null ? "—" : item.value }}
						</NTag>
					</div>
					<NButtonGroup size="tiny">
						<NButton :disabled="!canGoPrevious || isSearching" @click="goPreviousPage">
							上一页
						</NButton>
						<NButton
							:disabled="resultNextOffset === null || isSearching"
							@click="goNextPage"
						>
							下一页
						</NButton>
					</NButtonGroup>
				</div>
				<DataResultTable
					:columns="resultColumns"
					:data="resultRows"
					:loading="isSearching"
				/>
			</NCard>
		</div>
	</div>
</template>

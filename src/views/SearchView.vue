<script setup lang="ts">
import { Database, Search } from "lucide-vue-next"
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"

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
	connectProfile,
	openTable,
	setStatus,
	setError,
	clearMessages,
} = useWorkspace()

const route = useRoute()
const router = useRouter()

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
const onlyTableName = computed(() =>
	scopedTables.value.length === 1 ? (scopedTables.value[0]?.name ?? null) : null
)

function goToConnections() {
	const id = scopedProfileId.value
	void router.push(id ? `/connections/${id}` : "/")
}

async function connectScopedProfile() {
	const id = scopedProfileId.value
	if (!id) {
		goToConnections()
		return
	}
	clearMessages()
	await connectProfile(id)
}

async function selectOnlyTable() {
	const tableName = onlyTableName.value
	if (!tableName) {
		return
	}
	await onTableSelect(tableName)
}

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
			class="app-empty-state"
		>
			<div
				class="app-empty-state-icon"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="app-empty-state-title">选择连接</div>
				<div class="app-empty-state-description">
					请先在侧栏选择连接，再进行检索
				</div>
			</div>
			<NButton size="small" type="primary" @click="goToConnections">
				查看连接
			</NButton>
		</div>
		<div
			v-else-if="!scopedConnectionId"
			class="app-empty-state"
		>
			<div
				class="app-empty-state-icon app-empty-state-icon--warning"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="app-empty-state-title">尚未连接</div>
				<div class="app-empty-state-description">
					当前连接尚未建立，请在侧栏中点击连接
				</div>
			</div>
			<NButton size="small" type="primary" @click="connectScopedProfile">
				连接当前档案
			</NButton>
		</div>
		<div
			v-else-if="!hasActiveTable"
			class="app-empty-state"
		>
			<div
				class="app-empty-state-icon app-empty-state-icon--info"
			>
				<Search class="h-7 w-7" />
			</div>
			<div>
				<div class="app-empty-state-title">选择要检索的表</div>
				<div class="app-empty-state-description">
					从侧栏中选择表，或使用下方快速切换
				</div>
			</div>
			<NButton
				v-if="onlyTableName"
				size="small"
				type="primary"
				@click="selectOnlyTable"
			>
				使用 {{ onlyTableName }}
			</NButton>
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
				class="flex items-center justify-between rounded-md border border-[var(--app-rule)] bg-[var(--app-surface-elevated)] px-4 py-2 text-sm"
			>
				<div class="flex items-center gap-2 text-[var(--app-muted)]">
					<span>当前表：</span>
					<span class="font-medium text-[var(--app-ink)]">{{
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

			<NTabs v-model:value="activeTab" type="line" class="search-mode-tabs">
				<NTabPane name="filter" tab="过滤查询">
					<section class="query-builder">
						<header class="query-builder-header">
							<div>
								<h2 class="query-builder-title">过滤查询</h2>
								<p class="query-builder-subtitle">用 LanceDB filter 表达式读取行数据</p>
							</div>
							<NButton type="primary" :loading="isSearching" @click="runFilterQuery">
								查询
							</NButton>
						</header>
						<div class="query-grid">
							<label class="query-field query-field--span-2">
								<span>过滤表达式</span>
								<NInput v-model:value="filterExpression" placeholder="id > 10" />
							</label>
							<label class="query-field">
								<span>Limit</span>
								<NInputNumber v-model:value="filterLimit" :min="1" />
							</label>
							<label class="query-field">
								<span>Offset</span>
								<NInputNumber v-model:value="filterOffset" :min="0" />
							</label>
							<label class="query-field query-field--span-2">
								<span>列投影</span>
								<NSelect
									v-model:value="filterProjection"
									:options="columnOptions"
									multiple
									clearable
									placeholder="留空返回全部列"
								/>
							</label>
						</div>
					</section>
				</NTabPane>

				<NTabPane name="combined" tab="混合检索">
					<section class="query-builder">
						<header class="query-builder-header">
							<div>
								<h2 class="query-builder-title">混合检索</h2>
								<p class="query-builder-subtitle">同时提交全文查询和向量输入，结果由 RRF 融合</p>
							</div>
							<NButton type="primary" :loading="isSearching" @click="runCombinedQuery">
								混合检索
							</NButton>
						</header>
						<div class="query-grid query-grid--hybrid">
							<label class="query-field query-field--span-2">
								<span>全文查询</span>
								<NInput v-model:value="combinedQuery" placeholder="item 1" />
							</label>
							<label class="query-field query-field--span-2">
								<span>向量输入</span>
								<NInput
									v-model:value="combinedVectorText"
									placeholder="0.1, 0.2, 0.3"
								/>
							</label>
							<label class="query-field">
								<span>向量列</span>
								<NSelect
									v-model:value="combinedVectorColumn"
									:options="columnOptions"
									clearable
								/>
							</label>
							<label class="query-field">
								<span>全文列</span>
								<NSelect
									v-model:value="combinedColumns"
									:options="columnOptions"
									multiple
									clearable
								/>
							</label>
							<label class="query-field query-field--span-2">
								<span>Filter</span>
								<NInput v-model:value="combinedFilter" placeholder="id > 10" />
							</label>
							<label class="query-field query-field--compact">
								<span>Limit</span>
								<NInputNumber v-model:value="combinedLimit" :min="1" />
							</label>
							<label class="query-field query-field--compact">
								<span>Offset</span>
								<NInputNumber v-model:value="combinedOffset" :min="0" />
							</label>
							<label class="query-field query-field--compact">
								<span>nprobes</span>
								<NInputNumber v-model:value="combinedNprobes" :min="1" />
							</label>
							<label class="query-field query-field--compact">
								<span>refine</span>
								<NInputNumber v-model:value="combinedRefine" :min="1" />
							</label>
							<label class="query-field query-field--full">
								<span>列投影</span>
								<NSelect
									v-model:value="combinedProjection"
									:options="columnOptions"
									multiple
									clearable
									placeholder="留空返回全部列"
								/>
							</label>
						</div>
					</section>
				</NTabPane>

				<NTabPane name="vector" tab="向量检索">
					<section class="query-builder">
						<header class="query-builder-header">
							<div>
								<h2 class="query-builder-title">向量检索</h2>
								<p class="query-builder-subtitle">输入向量并选择向量列，返回最近邻结果</p>
							</div>
							<NButton type="primary" :loading="isSearching" @click="runVectorQuery">
								检索
							</NButton>
						</header>
						<div class="query-grid">
							<label class="query-field query-field--span-2">
								<span>向量输入</span>
								<NInput v-model:value="vectorText" placeholder="0.1, 0.2, 0.3" />
							</label>
							<label class="query-field">
								<span>向量列</span>
								<NSelect
									v-model:value="vectorColumn"
									:options="columnOptions"
									clearable
								/>
							</label>
							<label class="query-field query-field--compact">
								<span>Top K</span>
								<NInputNumber v-model:value="vectorTopK" :min="1" />
							</label>
							<label class="query-field">
								<span>Filter</span>
								<NInput v-model:value="vectorFilter" placeholder="id > 10" />
							</label>
							<label class="query-field query-field--compact">
								<span>Offset</span>
								<NInputNumber v-model:value="vectorOffset" :min="0" />
							</label>
							<label class="query-field query-field--compact">
								<span>nprobes</span>
								<NInputNumber v-model:value="vectorNprobes" :min="1" />
							</label>
							<label class="query-field query-field--compact">
								<span>refine</span>
								<NInputNumber v-model:value="vectorRefine" :min="1" />
							</label>
							<label class="query-field query-field--full">
								<span>列投影</span>
								<NSelect
									v-model:value="vectorProjection"
									:options="columnOptions"
									multiple
									clearable
									placeholder="留空返回全部列"
								/>
							</label>
						</div>
					</section>
				</NTabPane>

				<NTabPane name="fts" tab="全文检索">
					<section class="query-builder">
						<header class="query-builder-header">
							<div>
								<h2 class="query-builder-title">全文检索</h2>
								<p class="query-builder-subtitle">提交全文查询并限制检索列或结果范围</p>
							</div>
							<NButton type="primary" :loading="isSearching" @click="runFtsQuery">
								检索
							</NButton>
						</header>
						<div class="query-grid">
							<label class="query-field query-field--span-2">
								<span>查询文本</span>
								<NInput v-model:value="ftsQuery" placeholder="item 1" />
							</label>
							<label class="query-field">
								<span>索引列</span>
								<NSelect
									v-model:value="ftsColumns"
									:options="columnOptions"
									multiple
									clearable
								/>
							</label>
							<label class="query-field">
								<span>Filter</span>
								<NInput v-model:value="ftsFilter" placeholder="id > 10" />
							</label>
							<label class="query-field query-field--compact">
								<span>Limit</span>
								<NInputNumber v-model:value="ftsLimit" :min="1" />
							</label>
							<label class="query-field query-field--compact">
								<span>Offset</span>
								<NInputNumber v-model:value="ftsOffset" :min="0" />
							</label>
							<label class="query-field query-field--full">
								<span>列投影</span>
								<NSelect
									v-model:value="ftsProjection"
									:options="columnOptions"
									multiple
									clearable
									placeholder="留空返回全部列"
								/>
							</label>
						</div>
					</section>
				</NTabPane>
			</NTabs>

			<NAlert v-if="resultError" type="error" :bordered="false">
				{{ resultError }}
			</NAlert>

			<NCard size="small" title="结果" class="search-results-card">
				<div class="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs text-[var(--app-muted)]">
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

<style scoped>
.app-empty-state {
	display: flex;
	min-height: min(420px, calc(100vh - 220px));
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 14px;
	padding: 56px 20px;
	text-align: center;
}

.app-empty-state-icon {
	display: flex;
	width: 56px;
	height: 56px;
	align-items: center;
	justify-content: center;
	border-radius: 14px;
	background: var(--app-surface-panel-muted);
	color: var(--app-subtle);
}

.app-empty-state-icon--warning {
	background: var(--app-warning-soft);
	color: var(--app-warning);
}

.app-empty-state-icon--info {
	background: var(--app-accent-soft);
	color: var(--app-accent);
}

.app-empty-state-title {
	color: var(--app-ink);
	font-size: 16px;
	font-weight: 650;
}

.app-empty-state-description {
	margin-top: 4px;
	color: var(--app-muted);
	font-size: 14px;
}

.search-results-card {
	background: var(--app-surface-elevated);
	box-shadow: none;
}

.search-mode-tabs :deep(.n-tabs-pane-wrapper) {
	padding-top: 10px;
}

.query-builder {
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
	padding: 14px;
}

.query-builder-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 14px;
	margin-bottom: 14px;
	padding-bottom: 12px;
	border-bottom: 1px solid var(--app-rule);
}

.query-builder-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 15px;
	font-weight: 680;
	line-height: 1.3;
}

.query-builder-subtitle {
	margin: 4px 0 0;
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.4;
}

.query-grid {
	display: grid;
	grid-template-columns: repeat(4, minmax(0, 1fr));
	gap: 12px;
	align-items: end;
}

.query-grid--hybrid {
	grid-template-columns: repeat(4, minmax(0, 1fr));
}

.query-field {
	display: grid;
	min-width: 0;
	gap: 5px;
	color: var(--app-muted);
	font-size: 12px;
	font-weight: 620;
	line-height: 1.2;
}

.query-field--span-2 {
	grid-column: span 2;
}

.query-field--full {
	grid-column: 1 / -1;
}

.query-field--compact {
	min-width: 104px;
}

.query-field :deep(.n-input),
.query-field :deep(.n-input-number),
.query-field :deep(.n-base-selection) {
	width: 100%;
}

.query-field :deep(.n-input-number-button) {
	width: 24px;
}

:deep(label),
:deep(.text-slate-600),
:deep(.text-slate-500) {
	color: var(--app-muted);
}

@media (max-width: 1180px) {
	.query-grid,
	.query-grid--hybrid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}

@media (max-width: 700px) {
	.query-builder-header {
		flex-direction: column;
		align-items: stretch;
	}

	.query-grid,
	.query-grid--hybrid {
		grid-template-columns: 1fr;
	}

	.query-field--span-2,
	.query-field--full {
		grid-column: auto;
	}
}
</style>

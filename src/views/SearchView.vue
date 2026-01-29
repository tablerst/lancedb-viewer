<script setup lang="ts">
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, h, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"

import { useWorkspace } from "../composables/workspaceContext"
import type { SchemaDefinition } from "../ipc/v1"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "../lib/connectionKind"
import { formatCellValue, normalizeRow } from "../lib/formatters"
import {
	combinedSearchV1,
	ftsSearchV1,
	queryFilterV1,
	unwrapEnvelope,
	vectorSearchV1,
} from "../lib/tauriClient"

const {
	profiles,
	activeProfileId,
	connectionStates,
	connectProfile,
	refreshTables,
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
const scopedIsConnecting = computed(() => scopedConnection.value?.isConnecting.value ?? false)
const scopedIsRefreshing = computed(() => scopedConnection.value?.isRefreshing.value ?? false)
const scopedIsOpening = computed(() => scopedConnection.value?.isOpening.value ?? false)

const isLegacySearchRoute = computed(() => route.name === "search")

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

const tableHint = computed(() => scopedActiveTableName.value ?? "尚未选择表")
const hasActiveTable = computed(() => Boolean(scopedActiveTableId.value))

const tableOptions = computed<SelectOption[]>(() =>
	scopedTables.value.map((item) => ({ label: item.name, value: item.name }))
)

const selectedTableName = ref<string | null>(null)

watch(
	() => scopedActiveTableName.value,
	(next) => {
		if (!next) {
			return
		}
		if (selectedTableName.value !== next) {
			selectedTableName.value = next
		}
	},
	{ immediate: true }
)

async function gotoExplorer() {
	const id = scopedProfileId.value
	await router.push(id ? `/connections/${id}` : "/")
}

async function gotoCredentials() {
	const id = scopedProfileId.value
	await router.push(id ? `/connections/${id}/credentials` : "/vault/credentials")
}

async function switchToConnectionSearch() {
	const id = scopedProfileId.value
	if (!id) {
		return
	}
	await router.push(`/connections/${id}/search`)
}

async function connectCurrent() {
	const id = scopedProfileId.value
	if (!id) {
		return
	}
	await connectProfile(id)
}

async function refreshCurrentTables() {
	const id = scopedProfileId.value
	if (!id) {
		return
	}
	await refreshTables(id)
}

async function openSelectedTable() {
	const id = scopedProfileId.value
	const name = selectedTableName.value
	if (!id || !name) {
		setError("请先选择要打开的表")
		return
	}
	if (!scopedConnectionId.value) {
		setError("当前连接未连接")
		return
	}
	clearMessages()
	await openTable(id, name)
	setStatus(`已打开表：${name}`)
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
	const fields = resultSchema.value?.fields ?? scopedSchema.value?.fields ?? []
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

watch(scopedActiveTableId, () => {
	resetResults()
})

watch(scopedProfileId, () => {
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
	const tableId = scopedActiveTableId.value
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
	const tableId = scopedActiveTableId.value
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
	const tableId = scopedActiveTableId.value
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
	const tableId = scopedActiveTableId.value
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
			<div class="space-y-2 text-xs text-slate-500">
				<div class="flex flex-wrap items-start justify-between gap-2">
					<div class="min-w-0 space-y-1">
						<div class="flex items-center gap-2 text-slate-600">
							<span class="font-medium">当前连接</span>
							<template v-if="scopedProfile">
								<span class="max-w-[240px] truncate">{{ scopedProfile.name }}</span>
								<NTag
									size="small"
									:type="connectionKindTagType"
								>
									{{ connectionKindLabel ?? "Unknown" }}
								</NTag>
								<NTag v-if="scopedIsConnecting" size="small" type="warning">
									连接中
								</NTag>
								<NTag v-else-if="scopedConnectionId" size="small" type="success">
									已连接
								</NTag>
								<NTag v-else size="small" type="default">未连接</NTag>
							</template>
							<span v-else>尚未选择连接</span>
						</div>
						<div
							v-if="scopedProfile"
							class="max-w-[520px] truncate"
							:title="scopedProfile.uri"
						>
							{{ scopedProfile.uri }}
						</div>
					</div>

					<div class="flex shrink-0 flex-wrap gap-2">
						<NButton size="small" secondary @click="gotoExplorer">资源浏览</NButton>
						<NButton
							size="small"
							secondary
							:disabled="!scopedProfileId"
							@click="gotoCredentials"
						>
							凭证
						</NButton>
						<NButton
							v-if="isLegacySearchRoute && scopedProfileId"
							size="small"
							quaternary
							@click="switchToConnectionSearch"
						>
							使用连接级 URL
						</NButton>
					</div>
				</div>

				<div class="flex flex-wrap items-end justify-between gap-2">
					<div class="flex min-w-[280px] flex-1 items-end gap-2">
						<div class="flex-1">
							<label class="text-xs text-slate-500">检索表</label>
							<NSelect
								v-model:value="selectedTableName"
								:options="tableOptions"
								filterable
								clearable
								placeholder="选择要检索的表"
								size="small"
								:disabled="!scopedConnectionId"
								:loading="scopedIsRefreshing"
							/>
						</div>
						<NButton
							size="small"
							type="primary"
							:disabled="!scopedConnectionId || !selectedTableName"
							:loading="scopedIsOpening"
							@click="openSelectedTable"
						>
							打开
						</NButton>
					</div>

					<div class="flex shrink-0 flex-wrap gap-2">
						<NButton
							size="small"
							type="primary"
							:disabled="!scopedProfileId || Boolean(scopedConnectionId)"
							:loading="scopedIsConnecting"
							@click="connectCurrent"
						>
							连接
						</NButton>
						<NButton
							size="small"
							secondary
							:disabled="!scopedProfileId || !scopedConnectionId"
							:loading="scopedIsRefreshing"
							@click="refreshCurrentTables"
						>
							刷新表
						</NButton>
					</div>
				</div>

				<div class="text-slate-500">
					当前表：{{ tableHint }}。
					<span v-if="!hasActiveTable">请先选择并打开表。</span>
				</div>
			</div>
		</NCard>

		<NEmpty
			v-if="!scopedProfileId"
			description="请先在左侧选择连接，再进行检索。"
		>
			<template #extra>
				<NButton size="small" secondary @click="gotoExplorer">去资源浏览</NButton>
			</template>
		</NEmpty>
		<NEmpty
			v-else-if="!scopedConnectionId"
			description="当前连接尚未连接。点击“连接”后再选择表。"
		>
			<template #extra>
				<div class="flex flex-wrap justify-center gap-2">
					<NButton
						size="small"
						type="primary"
						:loading="scopedIsConnecting"
						@click="connectCurrent"
					>
						连接
					</NButton>
					<NButton size="small" secondary @click="gotoCredentials">
						检查凭证
					</NButton>
				</div>
			</template>
		</NEmpty>
		<NEmpty
			v-else-if="!hasActiveTable"
			description="请选择要检索的表，点击“打开”后开始查询。"
		>
			<template #extra>
				<NButton size="small" secondary @click="gotoExplorer">
					在资源浏览中打开
				</NButton>
			</template>
		</NEmpty>

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

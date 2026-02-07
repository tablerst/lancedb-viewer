<script setup lang="ts">
import type { DataTableColumns, SelectOption } from "naive-ui"
import { computed, ref, watch } from "vue"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { IndexDefinitionV1, IndexTypeV1 } from "../../ipc/v1"
import { createIndexV1, dropIndexV1, listIndexesV1, unwrapEnvelope } from "../../lib/tauriClient"
import { indexTypeLabels, indexTypeOptions, renderHeader } from "./explorerShared"

const { activeTableId, schema, setError, setStatus } = useWorkspace()

const hasActiveTable = computed(() => Boolean(activeTableId.value))
const allFieldNames = computed(() => schema.value?.fields.map((f) => f.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((n) => ({ label: n, value: n }))
)

// ── Index list ─────────────────────────────────────────

const indexes = ref<IndexDefinitionV1[]>([])
const isLoadingIndexes = ref(false)
const indexError = ref("")

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

const indexNameOptions = computed<SelectOption[]>(() =>
	indexes.value.map((idx) => ({ label: idx.name, value: idx.name }))
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
		const msg = error instanceof Error ? error.message : "获取索引失败"
		indexError.value = msg
		setError(msg)
	} finally {
		isLoadingIndexes.value = false
	}
}

// ── Create index ───────────────────────────────────────

const indexType = ref<IndexTypeV1>("auto")
const indexColumnsToCreate = ref<string[]>([])
const indexName = ref("")
const indexReplace = ref(true)
const { execute: execCreateIndex, isLoading: isCreatingIndex } = useCommand("创建索引失败")

async function submitCreateIndex() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}
	const columns = indexColumnsToCreate.value.map((c) => c.trim()).filter(Boolean)
	if (!columns.length) {
		setError("请选择索引列")
		return
	}
	const name = indexName.value.trim() || undefined
	await execCreateIndex(async () => {
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
	})
}

// ── Drop index ─────────────────────────────────────────

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

// ── Watchers ───────────────────────────────────────────

watch(
	activeTableId,
	() => {
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

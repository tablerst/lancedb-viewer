<script setup lang="ts">
import { LogicalPosition } from "@tauri-apps/api/dpi"
import { Menu, MenuItem } from "@tauri-apps/api/menu"
import { confirm } from "@tauri-apps/plugin-dialog"
import {
	ArrowLeftRight,
	ChevronRight,
	Database,
	History,
	ListTree,
	PenLine,
	Table2,
	TableProperties,
	Wrench,
} from "lucide-vue-next"
import { type Component, computed, h, nextTick, provide, readonly, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { SchemaFieldInput } from "../../ipc/v1"
import { createTableV1, dropTableV1, renameTableV1, unwrapEnvelope } from "../../lib/tauriClient"
import DataBrowseTab from "./DataBrowseTab.vue"
import DataWriteTab from "./DataWriteTab.vue"
import {
	createFieldDraft,
	DATA_REFRESH_KEY,
	type FieldDraft,
	fieldTypeOptions,
	isVectorType,
	TRIGGER_DATA_REFRESH_KEY,
	toFieldInput,
} from "./explorerShared"
import ImportExportTab from "./ImportExportTab.vue"
import IndexesTab from "./IndexesTab.vue"
import MaintenanceTab from "./MaintenanceTab.vue"
import SchemaTab from "./SchemaTab.vue"
import VersionsTab from "./VersionsTab.vue"

const {
	activeProfileId,
	activeProfile,
	connectionId,
	activeTableName,
	activeTableId,
	openTable,
	refreshTables,
	clearActiveTable,
	setError,
	setStatus,
	clearMessages,
} = useWorkspace()

const route = useRoute()
const router = useRouter()

// ── Data refresh (provide to children) ─────────────────

const dataRefreshCounter = ref(0)
provide(DATA_REFRESH_KEY, readonly(dataRefreshCounter))
provide(TRIGGER_DATA_REFRESH_KEY, () => {
	dataRefreshCounter.value++
})

// ── Tab management ─────────────────────────────────────

const VALID_TABS = [
	"schema",
	"data",
	"write",
	"import-export",
	"maintenance",
	"versions",
	"indexes",
] as const

const activeInnerTab = computed(() => {
	const tab = route.params.tab as string | undefined
	if (tab && (VALID_TABS as readonly string[]).includes(tab)) {
		return tab
	}
	return "schema"
})

const canManageTables = computed(() => Boolean(connectionId.value))
const connectionLabel = computed(() => activeProfile.value?.name ?? "未连接")

function renderTabLabel(icon: Component, label: string) {
	return () =>
		h("span", { class: "inline-flex items-center gap-1.5" }, [
			h(icon, { class: "h-3.5 w-3.5" }),
			label,
		])
}

function switchTab(tab: string | number) {
	const id = route.params.id as string
	const tableName = activeTableName.value
	if (id && tableName) {
		router.push({
			name: "table-tab",
			params: { id, tableName, tab: String(tab) },
		})
	}
}

let navigating = false

async function navigateToTable(tableName: string, tab = "schema") {
	const profileId = activeProfileId.value
	const id = route.params.id as string
	if (!profileId || !id) {
		return false
	}
	navigating = true
	try {
		if (activeTableName.value !== tableName) {
			await openTable(profileId, tableName)
		}
		await router.push({
			name: "table-tab",
			params: { id, tableName, tab },
		})
	} finally {
		navigating = false
	}
	return true
}

// ── Context menu ───────────────────────────────────────

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
					await navigateToTable(tableName)
				},
			}),
			await MenuItem.new({
				id: "data",
				text: "数据浏览",
				action: async () => {
					await navigateToTable(tableName, "data")
				},
			}),
			await MenuItem.new({
				id: "versions",
				text: "版本与时间旅行",
				action: async () => {
					await navigateToTable(tableName, "versions")
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
	await menu.popup(new LogicalPosition(event.clientX, event.clientY))
}

// ── Drop table ─────────────────────────────────────────

const { execute: execDropTable } = useCommand("删除表失败")

async function requestDropTable(tableName: string) {
	const shouldDrop = await confirm(`确定删除表 ${tableName} 吗？该操作不可撤销。`)
	if (!shouldDrop) {
		return
	}
	await dropTableByName(tableName)
}

async function dropTableByName(tableName: string) {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId || !tableName) {
		return
	}
	await execDropTable(async () => {
		unwrapEnvelope(await dropTableV1(currentConnectionId, tableName))
		setStatus(`已删除表 ${tableName}`)
		if (activeTableName.value === tableName) {
			clearActiveTable(profileId)
		}
		await refreshTables(profileId)
	})
}

function dropActiveTable() {
	const tableName = activeTableName.value
	if (!tableName) {
		return
	}
	void dropTableByName(tableName)
}

// ── Rename table ───────────────────────────────────────

const { execute: execRenameTable, isLoading: isRenamingTable } = useCommand("重命名表失败")
const renameTargetName = ref("")
const renameSourceTable = ref<string | null>(null)
const showRenameModal = ref(false)

async function openRenameModal(tableName: string) {
	if (!(await navigateToTable(tableName))) {
		return
	}
	renameSourceTable.value = tableName
	renameTargetName.value = tableName
	showRenameModal.value = true
}

async function submitRenameTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableName = renameSourceTable.value ?? activeTableName.value
	if (!profileId || !currentConnectionId || !tableName) {
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
	await execRenameTable(async () => {
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
		await refreshTables(profileId)
		await openTable(profileId, response.newTableName)
	})
}

watch(showRenameModal, (visible) => {
	if (!visible) {
		renameSourceTable.value = null
		renameTargetName.value = ""
	}
})

// ── Create table ───────────────────────────────────────

const showCreateTableModal = ref(false)
const createTableName = ref("")
const createFields = ref<FieldDraft[]>([createFieldDraft()])
const { execute: execCreateTable, isLoading: isCreatingTable } = useCommand("创建表失败")

watch(
	[() => route.query.action, () => canManageTables.value],
	async ([rawAction, canManage]) => {
		if (!canManage) {
			return
		}
		const action = Array.isArray(rawAction) ? rawAction[0] : rawAction
		if (action !== "create-table") {
			return
		}
		await nextTick()
		showCreateTableModal.value = true
		await router.replace({ query: { ...route.query, action: undefined } })
	},
	{ immediate: true }
)

function addCreateField() {
	createFields.value = [...createFields.value, createFieldDraft()]
}

function removeCreateField(index: number) {
	createFields.value = createFields.value.filter((_, idx) => idx !== index)
}

async function submitCreateTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	if (!profileId || !currentConnectionId) {
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
	await execCreateTable(async () => {
		unwrapEnvelope(await createTableV1(currentConnectionId, tableName, { fields }))
		setStatus(`已创建表 ${tableName}`)
		await refreshTables(profileId)
		await openTable(profileId, tableName)
		createTableName.value = ""
		createFields.value = [createFieldDraft()]
		showCreateTableModal.value = false
	})
}

// ── Watchers ───────────────────────────────────────────

watch(activeProfileId, () => {
	createTableName.value = ""
	createFields.value = [createFieldDraft()]
	showRenameModal.value = false
	renameSourceTable.value = null
	renameTargetName.value = ""
})

watch(activeTableId, () => {
	clearMessages()
	showRenameModal.value = false
	renameSourceTable.value = null
	renameTargetName.value = ""
})

// ── Route ↔ table sync ────────────────────────────────

watch(activeTableName, (newTable) => {
	if (navigating) return
	const id = route.params.id as string
	if (!id) return
	if (newTable) {
		if ((route.params.tableName as string | undefined) !== newTable) {
			router.replace({
				name: "table-tab",
				params: { id, tableName: newTable, tab: "schema" },
			})
		}
	} else if (route.params.tableName) {
		router.replace({
			name: "connection-explorer",
			params: { id },
		})
	}
})

watch(
	() => route.params.tableName as string | undefined,
	async (routeTable) => {
		if (navigating) return
		const profileId = activeProfileId.value
		if (!profileId) return
		if (routeTable && routeTable !== activeTableName.value) {
			await openTable(profileId, routeTable)
		} else if (!routeTable && activeTableName.value) {
			clearActiveTable(profileId)
		}
	}
)
</script>

<template>
	<div class="space-y-4">
		<div
			v-if="!canManageTables"
			class="flex flex-col items-center justify-center gap-3 py-20 text-center"
		>
			<div
				class="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="text-base font-semibold text-slate-700">连接数据库</div>
				<div class="mt-1 text-sm text-slate-500">
					请在侧栏选择连接并建立数据库连接
				</div>
			</div>
		</div>
		<div
			v-else-if="!activeTableName"
			class="flex flex-col items-center justify-center gap-3 py-20 text-center"
		>
			<div
				class="flex h-14 w-14 items-center justify-center rounded-2xl bg-sky-50 text-sky-400"
			>
				<Table2 class="h-7 w-7" />
			</div>
			<div>
				<div class="text-base font-semibold text-slate-700">选择数据表</div>
				<div class="mt-1 text-sm text-slate-500">
					从侧栏的表列表中选择一个表开始浏览
				</div>
			</div>
		</div>

		<template v-else>
			<div class="flex items-center gap-1.5 text-sm">
				<span class="text-slate-500">{{ connectionLabel }}</span>
				<ChevronRight class="h-3.5 w-3.5 text-slate-400" />
				<span
					class="cursor-context-menu font-medium text-slate-800"
					@contextmenu="showTableContextMenu(activeTableName!, $event)"
				>
					{{ activeTableName }}
				</span>
			</div>

			<NTabs :value="activeInnerTab" type="line" @update:value="switchTab">
				<NTabPane name="schema" :tab="renderTabLabel(TableProperties, 'Schema')">
					<SchemaTab @drop-table="dropActiveTable" />
				</NTabPane>
				<NTabPane name="data" :tab="renderTabLabel(Table2, '数据浏览')">
					<DataBrowseTab />
				</NTabPane>
				<NTabPane name="write" :tab="renderTabLabel(PenLine, '数据写入')">
					<DataWriteTab />
				</NTabPane>
				<NTabPane
					name="import-export"
					:tab="renderTabLabel(ArrowLeftRight, '导入导出')"
				>
					<ImportExportTab />
				</NTabPane>
				<NTabPane name="maintenance" :tab="renderTabLabel(Wrench, '维护')">
					<MaintenanceTab />
				</NTabPane>
				<NTabPane name="versions" :tab="renderTabLabel(History, '版本与时间旅行')">
					<VersionsTab />
				</NTabPane>
				<NTabPane name="indexes" :tab="renderTabLabel(ListTree, '索引管理')">
					<IndexesTab />
				</NTabPane>
			</NTabs>
		</template>

		<NModal
			v-model:show="showRenameModal"
			:mask-closable="!isRenamingTable"
			:close-on-esc="!isRenamingTable"
		>
			<NCard
				size="small"
				title="重命名表"
				class="w-[420px]"
				:closable="!isRenamingTable"
				:bordered="false"
				@close="showRenameModal = false"
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

		<NModal
			v-model:show="showCreateTableModal"
			:mask-closable="!isCreatingTable"
			:close-on-esc="!isCreatingTable"
		>
			<NCard
				size="small"
				title="创建表"
				class="w-[760px] max-w-[calc(100vw-40px)]"
				:closable="!isCreatingTable"
				:bordered="false"
				@close="showCreateTableModal = false"
			>
				<div class="grid gap-3 xl:grid-cols-6">
					<div class="xl:col-span-2">
						<label class="text-sm font-medium text-slate-600">表名</label>
						<NInput v-model:value="createTableName" placeholder="new_table" />
					</div>
					<div class="xl:col-span-4 flex items-end justify-end gap-2">
						<NButton
							quaternary
							:disabled="isCreatingTable"
							@click="showCreateTableModal = false"
						>
							取消
						</NButton>
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
							:disabled="isCreatingTable"
							@click="removeCreateField(index)"
						>
							移除
						</NButton>
					</div>
				</div>
			</NCard>
		</NModal>
	</div>
</template>

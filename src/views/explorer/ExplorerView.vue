<script setup lang="ts">
import { LogicalPosition } from "@tauri-apps/api/dpi"
import { Menu, MenuItem } from "@tauri-apps/api/menu"
import { confirm } from "@tauri-apps/plugin-dialog"
import {
	ChevronDown,
	ChevronRight,
	Database,
	History,
	ListTree,
	Table2,
	TableProperties,
} from "lucide-vue-next"
import type { DropdownMixedOption } from "naive-ui/lib/dropdown/src/interface"
import { type Component, computed, h, nextTick, provide, readonly, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import {
	dropTableV1,
	getTableVersionV1,
	listIndexesV1,
	unwrapEnvelope,
} from "../../lib/tauriClient"
import CreateTableDialog from "./CreateTableDialog.vue"
import DataTab from "./DataTab.vue"
import ExportDialog from "./ExportDialog.vue"
import { DATA_REFRESH_KEY, TRIGGER_DATA_REFRESH_KEY } from "./explorerShared"
import ImportDialog from "./ImportDialog.vue"
import IndexesTab from "./IndexesTab.vue"
import MaintenanceDialog from "./MaintenanceDialog.vue"
import RenameTableDialog from "./RenameTableDialog.vue"
import SchemaTab from "./SchemaTab.vue"
import VersionsTab from "./VersionsTab.vue"

const {
	activeProfileId,
	activeProfile,
	connectionId,
	tables,
	activeTableName,
	activeTableId,
	schema,
	isConnecting,
	isRefreshing,
	connectProfile,
	openTable,
	refreshTables,
	clearActiveTable,
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

const VALID_TABS = ["schema", "data", "versions", "indexes"] as const

const activeInnerTab = computed(() => {
	const tab = route.params.tab as string | undefined
	if (tab && (VALID_TABS as readonly string[]).includes(tab)) {
		return tab
	}
	return "schema"
})

// ── Dialog state (import / export / maintenance) ───────

const showImportDialog = ref(false)
const showExportDialog = ref(false)
const showMaintenanceDialog = ref(false)

// ── Table summary info ─────────────────────────────────

const tableSummaryVersion = ref<number | null>(null)
const tableSummaryIndexCount = ref<number | null>(null)
const fieldCount = computed(() => schema.value?.fields.length ?? 0)

async function loadTableSummary() {
	const tableId = activeTableId.value
	if (!tableId) return
	try {
		const versionResponse = unwrapEnvelope(await getTableVersionV1({ tableId }))
		tableSummaryVersion.value = versionResponse.version
	} catch {
		tableSummaryVersion.value = null
	}
	try {
		const indexResponse = unwrapEnvelope(await listIndexesV1(tableId))
		tableSummaryIndexCount.value = indexResponse.indexes.length
	} catch {
		tableSummaryIndexCount.value = null
	}
}

const canManageTables = computed(() => Boolean(connectionId.value))
const connectionLabel = computed(() => activeProfile.value?.name ?? "未连接")
const tableDropdownOptions = computed<DropdownMixedOption[]>(() =>
	tables.value.map((table) => ({ label: table.name, key: table.name }))
)

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

function selectBreadcrumbTable(key: string | number) {
	if (typeof key !== "string" || key === activeTableName.value) {
		return
	}
	clearMessages()
	void navigateToTable(key, activeInnerTab.value)
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

function connectActiveProfile() {
	const profileId = activeProfileId.value
	if (!profileId) {
		return
	}
	void connectProfile(profileId)
}

function refreshActiveTables() {
	const profileId = activeProfileId.value
	if (!profileId) {
		return
	}
	void refreshTables(profileId)
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
				id: "import",
				text: "导入数据…",
				action: async () => {
					if (await navigateToTable(tableName, "data")) {
						showImportDialog.value = true
					}
				},
			}),
			await MenuItem.new({
				id: "export",
				text: "导出数据…",
				action: async () => {
					if (await navigateToTable(tableName, "data")) {
						showExportDialog.value = true
					}
				},
			}),
			await MenuItem.new({
				id: "maintenance",
				text: "维护（Compact / Vacuum）…",
				action: async () => {
					if (await navigateToTable(tableName)) {
						showMaintenanceDialog.value = true
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

// ── Rename table (dialog) ──────────────────────────────

const showRenameModal = ref(false)
const renameSourceTable = ref<string | null>(null)

async function openRenameModal(tableName: string) {
	if (!(await navigateToTable(tableName))) {
		return
	}
	renameSourceTable.value = tableName
	showRenameModal.value = true
}

// ── Create table (dialog) ──────────────────────────────

const showCreateTableModal = ref(false)

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

// ── Watchers ───────────────────────────────────────────

watch(activeProfileId, () => {
	showRenameModal.value = false
	renameSourceTable.value = null
	showCreateTableModal.value = false
})

watch(activeTableId, () => {
	clearMessages()
	showRenameModal.value = false
	renameSourceTable.value = null
	tableSummaryVersion.value = null
	tableSummaryIndexCount.value = null
	if (activeTableId.value) {
		void loadTableSummary()
	}
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
	<div class="flex h-full flex-col">
		<div
			v-if="!canManageTables"
			class="explorer-empty"
		>
			<div
				class="explorer-empty-icon"
			>
				<Database class="h-7 w-7" />
			</div>
			<div>
				<div class="explorer-empty-title">连接数据库</div>
				<div class="explorer-empty-description">
					请在侧栏选择连接并建立数据库连接
				</div>
			</div>
			<NButton
				v-if="activeProfileId"
				size="small"
				type="primary"
				:loading="isConnecting"
				@click="connectActiveProfile"
			>
				连接当前档案
			</NButton>
		</div>
		<div
			v-else-if="!activeTableName"
			class="explorer-empty"
		>
			<div
				class="explorer-empty-icon explorer-empty-icon--info"
			>
				<Table2 class="h-7 w-7" />
			</div>
			<div>
				<div class="explorer-empty-title">选择数据表</div>
				<div class="explorer-empty-description">
					从侧栏的表列表中选择一个表开始浏览
				</div>
			</div>
			<div class="flex flex-wrap justify-center gap-2">
				<NButton size="small" :loading="isRefreshing" @click="refreshActiveTables">
					刷新表列表
				</NButton>
				<NButton size="small" type="primary" @click="showCreateTableModal = true">
					创建表
				</NButton>
			</div>
		</div>

		<template v-else>
			<!-- Sticky breadcrumb + tabs header -->
			<div class="sticky top-0 z-20 bg-[var(--app-surface)] pb-1">
				<div class="explorer-breadcrumb py-2 text-sm">
					<span class="text-[var(--app-muted)]">{{ connectionLabel }}</span>
					<ChevronRight class="h-3.5 w-3.5 text-[var(--app-subtle)]" />
					<NDropdown
						v-if="tableDropdownOptions.length > 1"
						trigger="click"
						placement="bottom-start"
						:options="tableDropdownOptions"
						:show-arrow="false"
						@select="selectBreadcrumbTable"
					>
						<button
							type="button"
							class="breadcrumb-table-trigger"
							:aria-label="`切换表，当前为 ${activeTableName}`"
							@contextmenu="showTableContextMenu(activeTableName!, $event)"
						>
							<span class="breadcrumb-table-name">{{ activeTableName }}</span>
							<ChevronDown class="breadcrumb-table-caret" aria-hidden="true" />
						</button>
					</NDropdown>
					<span
						v-else
						class="breadcrumb-table-trigger breadcrumb-table-trigger--single"
						@contextmenu="showTableContextMenu(activeTableName!, $event)"
					>
						<span class="breadcrumb-table-name">{{ activeTableName }}</span>
					</span>
					<span class="mx-1 text-[var(--app-rule-strong)]">|</span>
					<span class="explorer-breadcrumb-summary">
						<span>{{ fieldCount }} 列</span>
						<span v-if="tableSummaryVersion !== null">v{{ tableSummaryVersion }}</span>
						<span v-if="tableSummaryIndexCount !== null">
							{{ tableSummaryIndexCount }} 索引
						</span>
					</span>
				</div>

				<NTabs
					:value="activeInnerTab"
					type="line"
					display-directive="if"
					@update:value="switchTab"
				>
					<template #suffix>
						<div class="flex items-center gap-1">
							<NButton
								quaternary
								size="tiny"
								@click="showImportDialog = true"
							>
								导入
							</NButton>
							<NButton
								quaternary
								size="tiny"
								@click="showExportDialog = true"
							>
								导出
							</NButton>
							<NButton
								quaternary
								size="tiny"
								@click="showMaintenanceDialog = true"
							>
								维护
							</NButton>
						</div>
					</template>
					<NTab name="schema" :tab="renderTabLabel(TableProperties, 'Schema')" />
					<NTab name="data" :tab="renderTabLabel(Table2, '数据')" />
					<NTab name="indexes" :tab="renderTabLabel(ListTree, '索引')" />
					<NTab name="versions" :tab="renderTabLabel(History, '版本')" />
				</NTabs>
			</div>

			<!-- Scrollable tab content -->
			<div class="min-h-0 flex-1 overflow-y-auto pt-2">
				<SchemaTab
					v-if="activeInnerTab === 'schema'"
					@drop-table="dropActiveTable"
				/>
				<DataTab
					v-else-if="activeInnerTab === 'data'"
					@request-export="showExportDialog = true"
				/>
				<IndexesTab v-else-if="activeInnerTab === 'indexes'" />
				<VersionsTab v-else-if="activeInnerTab === 'versions'" />
			</div>
		</template>

		<!-- Dialogs -->
		<ImportDialog v-model:show="showImportDialog" />
		<ExportDialog v-model:show="showExportDialog" />
		<MaintenanceDialog v-model:show="showMaintenanceDialog" />
		<RenameTableDialog
			v-model:show="showRenameModal"
			:table-name="renameSourceTable"
		/>
		<CreateTableDialog v-model:show="showCreateTableModal" />
	</div>
</template>

<style scoped>
.explorer-breadcrumb {
	display: flex;
	min-width: 0;
	align-items: center;
	gap: 6px;
}

.breadcrumb-table-trigger {
	display: inline-flex;
	min-width: 0;
	max-width: min(340px, 44vw);
	align-items: center;
	gap: 5px;
	padding: 2px 5px;
	border: 0;
	border-radius: var(--app-radius-sm);
	background: transparent;
	color: var(--app-ink);
	cursor: pointer;
	font: inherit;
	font-weight: 650;
	line-height: 1.35;
}

.breadcrumb-table-trigger:hover {
	background: var(--app-surface-panel-muted);
}

.breadcrumb-table-trigger:focus-visible {
	outline: 2px solid var(--app-accent);
	outline-offset: 2px;
}

.breadcrumb-table-trigger:active {
	transform: translateY(1px);
}

.breadcrumb-table-trigger:disabled {
	cursor: default;
}

.breadcrumb-table-trigger--single:hover,
.breadcrumb-table-trigger--single:disabled {
	background: transparent;
}

.breadcrumb-table-trigger--single {
	cursor: context-menu;
}

.breadcrumb-table-name {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.breadcrumb-table-caret {
	width: 13px;
	height: 13px;
	flex: 0 0 auto;
	color: var(--app-subtle);
}

.explorer-breadcrumb-summary {
	display: inline-flex;
	flex: 0 0 auto;
	align-items: center;
	gap: 12px;
	color: var(--app-muted);
	font-size: 12px;
}

.explorer-empty {
	display: flex;
	min-height: min(420px, calc(100vh - 220px));
	flex: 1;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 14px;
	padding: 56px 20px;
	text-align: center;
}

.explorer-empty-icon {
	display: flex;
	width: 56px;
	height: 56px;
	align-items: center;
	justify-content: center;
	border-radius: 14px;
	background: var(--app-surface-panel-muted);
	color: var(--app-subtle);
}

.explorer-empty-icon--info {
	background: var(--app-accent-soft);
	color: var(--app-accent);
}

.explorer-empty-title {
	color: var(--app-ink);
	font-size: 16px;
	font-weight: 650;
}

.explorer-empty-description {
	margin-top: 4px;
	color: var(--app-muted);
	font-size: 14px;
}
</style>

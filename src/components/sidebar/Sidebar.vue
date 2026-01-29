<script setup lang="ts">
import { listen } from "@tauri-apps/api/event"
import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
import { getCurrentWindow } from "@tauri-apps/api/window"
import {
	ChevronLeft,
	ChevronRight,
	Filter,
	Key,
	Pencil,
	Plug,
	Plus,
	RefreshCcw,
	Table,
	Trash2,
} from "lucide-vue-next"
import { useDialog } from "naive-ui"
import type { DropdownMixedOption } from "naive-ui/lib/dropdown/src/interface"
import { computed, h, onBeforeUnmount, onMounted, ref } from "vue"
import { useRouter } from "vue-router"

import type { ConnectionState } from "../../composables/useConnection"
import { useWorkspace } from "../../composables/workspaceContext"
import type { AuthDescriptor } from "../../ipc/v1"
import type { ConnectionKind } from "../../lib/connectionKind"
import { getConnectionKind, getConnectionKindLabel } from "../../lib/connectionKind"
import type { StoredProfile } from "../../models/profile"
import ConnectionItem from "./ConnectionItem.vue"

const props = defineProps<{
	profiles: StoredProfile[]
	activeProfileId: string | null
	connectionStates: Record<string, ConnectionState>
	onSelectProfile: (profileId: string) => void | Promise<void>
	onConnectProfile: (profileId: string) => void | Promise<void>
	onDisconnectProfile: (profileId: string) => void | Promise<void>
	onRefreshTables: (profileId: string) => void | Promise<void>
	onOpenTable: (profileId: string, tableName: string) => void | Promise<void>
}>()

const isCollapsed = ref(false)
const router = useRouter()
const dialog = useDialog()

const {
	profileForm,
	addProfile,
	updateProfile,
	deleteProfile,
	resetConnection,
	setStatus,
	setError,
	clearMessages,
} = useWorkspace()

type CreateProfilePayload = {
	name: string
	uri: string
	storageOptionsJson?: string
	auth?: AuthDescriptor
}

type UpdateProfilePayload = {
	id: string
	name: string
	uri: string
	storageOptionsJson?: string
	auth?: AuthDescriptor
}

const createDialogLabel = "dialog-new-connection"
const editDialogLabel = "dialog-edit-connection"
let unlistenCreateProfile: (() => void) | null = null
let unlistenUpdateProfile: (() => void) | null = null

type ProfileFilterKind = "all" | ConnectionKind
const filterKind = ref<ProfileFilterKind>("all")

const expandedWidth = "clamp(240px, 26vw, 340px)"
const collapsedWidth = "clamp(64px, 8vw, 84px)"
const collapsedItemSize = 116

const filteredProfiles = computed(() => {
	if (filterKind.value === "all") {
		return props.profiles
	}
	return props.profiles.filter((profile) => getConnectionKind(profile.uri) === filterKind.value)
})

const shouldVirtualize = computed(() => isCollapsed.value && filteredProfiles.value.length > 12)

const sidebarWidth = computed(() => (isCollapsed.value ? collapsedWidth : expandedWidth))
const virtualItemSize = computed(() => (isCollapsed.value ? collapsedItemSize : 92))

onMounted(async () => {
	try {
		unlistenCreateProfile = await listen<CreateProfilePayload>(
			"profiles:create",
			async ({ payload }) => {
				clearMessages()
				profileForm.value = {
					name: payload.name ?? "",
					uri: payload.uri ?? "",
					storageOptionsJson: payload.storageOptionsJson?.trim()
						? payload.storageOptionsJson
						: "{}",
					auth: payload.auth ?? { type: "none" },
				}
				await addProfile()
			}
		)
		unlistenUpdateProfile = await listen<UpdateProfilePayload>(
			"profiles:update",
			async ({ payload }) => {
				clearMessages()
				if (!payload.id) {
					setError("未找到需要更新的连接档案")
					return
				}
				await updateProfile({
					id: payload.id,
					name: payload.name ?? "",
					uri: payload.uri ?? "",
					storageOptionsJson: payload.storageOptionsJson?.trim()
						? payload.storageOptionsJson
						: "{}",
					auth: payload.auth,
				})
			}
		)
	} catch (error) {
		const message = error instanceof Error ? error.message : "注册新建连接监听失败"
		setError(message)
	}
})

onBeforeUnmount(() => {
	if (unlistenCreateProfile) {
		unlistenCreateProfile()
		unlistenCreateProfile = null
	}
	if (unlistenUpdateProfile) {
		unlistenUpdateProfile()
		unlistenUpdateProfile = null
	}
})

function toggleCollapse() {
	isCollapsed.value = !isCollapsed.value
}

async function openCreateDialog() {
	clearMessages()
	setStatus("已打开新建连接")
	try {
		const existing = await WebviewWindow.getByLabel(createDialogLabel)
		if (existing) {
			await existing.setFocus()
			return
		}
		const parent = getCurrentWindow()
		const dialog = new WebviewWindow(createDialogLabel, {
			url: "/#/dialog/new-connection",
			title: "新建连接",
			width: 520,
			height: 520,
			center: true,
			resizable: false,
			minimizable: false,
			maximizable: false,
			parent,
		})
		dialog.once("tauri://error", () => {
			setError("打开新建连接窗口失败")
		})
	} catch (error) {
		const message = error instanceof Error ? error.message : "打开新建连接窗口失败"
		setError(message)
	}
}

async function openEditDialog(profileId: string) {
	clearMessages()
	setStatus("已打开编辑连接")
	try {
		const existing = await WebviewWindow.getByLabel(editDialogLabel)
		if (existing) {
			await existing.close()
		}
		const parent = getCurrentWindow()
		const dialog = new WebviewWindow(editDialogLabel, {
			url: `/#/dialog/edit-connection?profileId=${encodeURIComponent(profileId)}`,
			title: "编辑连接",
			width: 520,
			height: 540,
			center: true,
			resizable: false,
			minimizable: false,
			maximizable: false,
			parent,
		})
		dialog.once("tauri://error", () => {
			setError("打开编辑连接窗口失败")
		})
	} catch (error) {
		const message = error instanceof Error ? error.message : "打开编辑连接窗口失败"
		setError(message)
	}
}

async function handleDeleteProfile(profileId: string) {
	clearMessages()
	try {
		resetConnection(profileId)
		await deleteProfile(profileId)
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除连接档案失败"
		setError(message)
	}
}

async function selectAndOpenTable(profileId: string, tableName: string) {
	await props.onSelectProfile(profileId)
	await props.onOpenTable(profileId, tableName)
}

function getConnectionFlags(profileId: string) {
	const state = props.connectionStates[profileId] ?? null
	const isConnected = Boolean(state?.connectionId?.value)
	const isConnecting = state?.isConnecting?.value ?? false
	const isDisconnecting = state?.isDisconnecting?.value ?? false
	const isRefreshing = state?.isRefreshing?.value ?? false
	return {
		isConnected,
		isConnecting,
		isDisconnecting,
		isRefreshing,
	}
}

const contextMenuVisible = ref(false)
const contextMenuX = ref<number | null>(null)
const contextMenuY = ref<number | null>(null)
const contextMenuProfile = ref<StoredProfile | null>(null)

function hideContextMenu() {
	contextMenuVisible.value = false
}

function handleContextMenuUpdateShow(value: boolean) {
	contextMenuVisible.value = value
}

function showProfileContextMenu(profile: StoredProfile, event: MouseEvent) {
	event.preventDefault()
	contextMenuProfile.value = profile
	contextMenuX.value = event.clientX
	contextMenuY.value = event.clientY
	contextMenuVisible.value = true
}

const contextMenuOptions = computed<DropdownMixedOption[]>(() => {
	const profile = contextMenuProfile.value
	if (!profile) {
		return []
	}
	const { isConnected, isConnecting, isDisconnecting, isRefreshing } = getConnectionFlags(
		profile.id
	)
	const connectLabel = isConnected ? "重连" : "连接"

	const canConnect = !isConnecting && !isDisconnecting
	const canDisconnect = isConnected && !isDisconnecting
	const canRefresh = isConnected && !isRefreshing && !isDisconnecting && !isConnecting
	const canCreateTable = isConnected && !isDisconnecting && !isConnecting

	return [
		{
			key: "connect",
			label: connectLabel,
			disabled: !canConnect,
			icon: () => h(Plug, { class: "h-4 w-4" }),
		},
		{
			key: "disconnect",
			label: "断开",
			disabled: !canDisconnect,
			icon: () => h(Plug, { class: "h-4 w-4" }),
		},
		{
			key: "refresh",
			label: "刷新表",
			disabled: !canRefresh,
			icon: () => h(RefreshCcw, { class: "h-4 w-4" }),
		},
		{
			key: "create-table",
			label: "创建表…",
			disabled: !canCreateTable,
			icon: () => h(Table, { class: "h-4 w-4" }),
			props: {
				title: canCreateTable ? "创建新表" : "仅已连接时可用",
			},
		},
		{ type: "divider", key: "d-actions" },
		{ key: "credentials", label: "凭证…", icon: () => h(Key, { class: "h-4 w-4" }) },
		{ key: "edit", label: "编辑…", icon: () => h(Pencil, { class: "h-4 w-4" }) },
		{ type: "divider", key: "d-danger" },
		{
			key: "delete",
			label: () => h("span", { class: "font-medium text-rose-600" }, "删除连接…"),
			icon: () => h(Trash2, { class: "h-4 w-4" }),
			props: { class: "text-rose-600" },
		},
	]
})

async function handleContextMenuSelect(key: string | number) {
	const profile = contextMenuProfile.value
	hideContextMenu()
	if (!profile) {
		return
	}

	try {
		switch (String(key)) {
			case "connect":
				await props.onConnectProfile(profile.id)
				break
			case "disconnect":
				await props.onDisconnectProfile(profile.id)
				break
			case "refresh":
				await props.onRefreshTables(profile.id)
				break
			case "create-table":
				await props.onSelectProfile(profile.id)
				await router.push({
					path: `/connections/${profile.id}`,
					query: { action: "create-table" },
				})
				break
			case "credentials":
				await router.push(`/connections/${profile.id}/credentials`)
				break
			case "edit":
				await openEditDialog(profile.id)
				break
			case "delete":
				dialog.warning({
					title: "删除连接",
					content: `确定删除连接档案 ${profile.name} 吗？该操作不可撤销。`,
					positiveText: "删除",
					negativeText: "取消",
					onPositiveClick: async () => {
						await handleDeleteProfile(profile.id)
					},
				})
				break
			default:
				break
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : "执行操作失败"
		setError(message)
	}
}
</script>

<template>
	<aside
		class="relative flex h-full shrink-0 flex-col border-r border-slate-200 bg-white transition-[width] duration-200 ease-out"
		:style="{ width: sidebarWidth }"
	>
		<NDropdown
			:show="contextMenuVisible"
			:options="contextMenuOptions"
			:x="contextMenuX ?? undefined"
			:y="contextMenuY ?? undefined"
			trigger="manual"
			placement="bottom-start"
			:show-arrow="false"
			@select="handleContextMenuSelect"
			@clickoutside="hideContextMenu"
			@update:show="handleContextMenuUpdateShow"
		>
			<!--
				Naive NDropdown 复用 NPopover：默认插槽会被当作 trigger。
				必须且只能提供 1 个子节点，否则会触发 vueuc/follower 的运行时错误。
				这里放一个不可见锚点即可（实际弹出位置由 x/y 控制）。
			-->
			<div
				aria-hidden="true"
				class="pointer-events-none fixed left-0 top-0 h-0 w-0 overflow-hidden"
			/>
		</NDropdown>
		<div class="absolute right-0 top-1/2 z-10 -translate-y-1/2 translate-x-1/2">
			<NButton
				size="tiny"
				quaternary
				circle
				:aria-label="isCollapsed ? '展开侧边栏' : '收起侧边栏'"
				:title="isCollapsed ? '展开侧边栏' : '收起侧边栏'"
				class="bg-white shadow-sm ring-1 ring-slate-200 transition hover:ring-slate-300"
				@click="toggleCollapse"
			>
				<ChevronRight v-if="isCollapsed" class="h-4 w-4" />
				<ChevronLeft v-else class="h-4 w-4" />
			</NButton>
		</div>

		<div
			class="flex items-center pt-4"
			:class="isCollapsed ? 'justify-center px-3' : 'justify-between px-4'"
		>
			<div v-if="!isCollapsed" class="min-w-0">
				<div class="text-sm font-semibold text-slate-900">连接</div>
				<div class="text-xs text-slate-500">选择连接后浏览表 / 配置凭证</div>
			</div>

			<div
				class="flex items-center gap-2"
				:class="isCollapsed ? 'flex-col justify-center' : 'flex-row'"
			>
				<NPopover trigger="click" placement="bottom-start">
					<template #trigger>
						<NButton size="small" quaternary>
							<Filter class="h-4 w-4" />
							<span v-if="!isCollapsed" class="ml-2">筛选</span>
						</NButton>
					</template>
					<div class="w-56 space-y-2 p-2">
						<div class="text-xs font-medium text-slate-700">连接类型</div>
						<NRadioGroup v-model:value="filterKind" size="small">
							<div class="filter-radio-grid grid grid-cols-2 gap-2 p-0.5">
								<NRadioButton value="all">全部</NRadioButton>
								<NRadioButton value="local">{{ getConnectionKindLabel("local") }}</NRadioButton>
								<NRadioButton value="s3">{{ getConnectionKindLabel("s3") }}</NRadioButton>
								<NRadioButton value="remote">{{ getConnectionKindLabel("remote") }}</NRadioButton>
								<NRadioButton value="gcs">{{ getConnectionKindLabel("gcs") }}</NRadioButton>
								<NRadioButton value="azure">{{ getConnectionKindLabel("azure") }}</NRadioButton>
							</div>
						</NRadioGroup>
					</div>
				</NPopover>

				<NButton size="small" type="primary" @click.stop="openCreateDialog">
					<Plus class="h-4 w-4" />
					<span v-if="!isCollapsed" class="ml-2">新建</span>
				</NButton>
			</div>
		</div>

		<div class="mt-4 flex min-h-0 flex-1 flex-col overflow-hidden px-3 pb-4">
			<div class="min-h-0 flex-1 overflow-y-auto">
				<NEmpty v-if="!filteredProfiles.length" description="暂无连接档案" />
				<template v-else>
					<NVirtualList
						v-if="shouldVirtualize"
						:items="filteredProfiles"
						:item-size="virtualItemSize"
					>
						<template #default="{ item }">
							<div class="pb-3">
								<ConnectionItem
									:key="item.id"
									:profile="item"
									:state="connectionStates[item.id] ?? null"
									:selected="item.id === activeProfileId"
									:collapsed="isCollapsed"
									@select="onSelectProfile(item.id)"
									@open-table="(name) => selectAndOpenTable(item.id, name)"
									@open-menu="(event) => showProfileContextMenu(item, event)"
								/>
							</div>
						</template>
					</NVirtualList>
					<div v-else class="space-y-3">
						<ConnectionItem
							v-for="profile in filteredProfiles"
							:key="profile.id"
							:profile="profile"
							:state="connectionStates[profile.id] ?? null"
							:selected="profile.id === activeProfileId"
							:collapsed="isCollapsed"
							@select="onSelectProfile(profile.id)"
							@open-table="(name) => selectAndOpenTable(profile.id, name)"
							@open-menu="(event) => showProfileContextMenu(profile, event)"
						/>
					</div>
				</template>
			</div>
		</div>

	</aside>
</template>

<style scoped>
.filter-radio-grid {
	overflow: visible;	
}

.filter-radio-grid :deep(.n-radio-button) {
	border-radius: 8px !important;	
}

.filter-radio-grid :deep(.n-radio-button:not(:first-child)) {
	border-left-width: 1px !important;
}

.filter-radio-grid :deep(.n-radio-button__content) {
	justify-content: center;
	width: 100%;
}
</style>

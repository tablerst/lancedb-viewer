<script setup lang="ts">
import { listen } from "@tauri-apps/api/event"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
import { ChevronLeft, ChevronRight, Database, Filter, Plus } from "lucide-vue-next"
import { computed, onBeforeUnmount, onMounted, ref } from "vue"

import type { ConnectionState } from "../../composables/useConnection"
import { useWorkspace } from "../../composables/workspaceContext"
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
	onRefreshTables: (profileId: string) => void | Promise<void>
	onOpenTable: (profileId: string, tableName: string) => void | Promise<void>
}>()

const isCollapsed = ref(false)

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
}

type UpdateProfilePayload = {
	id: string
	name: string
	uri: string
	storageOptionsJson?: string
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

async function selectAndConnect(profileId: string) {
	await props.onSelectProfile(profileId)
	await props.onConnectProfile(profileId)
}

async function selectAndRefresh(profileId: string) {
	await props.onSelectProfile(profileId)
	await props.onRefreshTables(profileId)
}

async function selectAndOpenTable(profileId: string, tableName: string) {
	await props.onSelectProfile(profileId)
	await props.onOpenTable(profileId, tableName)
}
</script>

<template>
	<aside
		class="relative flex h-full shrink-0 flex-col border-r border-slate-200 bg-white transition-[width] duration-200 ease-out"
		:style="{ width: sidebarWidth }"
	>
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
			<div class="flex items-center gap-3">
				<div
					class="flex h-9 w-9 items-center justify-center rounded-lg bg-sky-500 text-white"
				>
					<Database class="h-5 w-5" />
				</div>
				<div v-if="!isCollapsed" class="min-w-0">
					<div class="text-sm font-semibold text-slate-900">LanceDB Studio</div>
					<div class="text-xs text-slate-500">JSON-first IPC</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<NTag v-if="!isCollapsed" size="small" type="info">v1</NTag>
			</div>
		</div>

		<div class="mt-4 px-3" :class="isCollapsed ? 'px-2' : 'px-3'">
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
									@connect="selectAndConnect(item.id)"
									@refresh="selectAndRefresh(item.id)"
									@open-table="(name) => selectAndOpenTable(item.id, name)"
									@edit="openEditDialog(item.id)"
									@delete="handleDeleteProfile(item.id)"
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
							@connect="selectAndConnect(profile.id)"
							@refresh="selectAndRefresh(profile.id)"
							@open-table="(name) => selectAndOpenTable(profile.id, name)"
							@edit="openEditDialog(profile.id)"
							@delete="handleDeleteProfile(profile.id)"
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

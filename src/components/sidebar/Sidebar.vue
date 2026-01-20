<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog"
import { Database, Filter, FolderOpen, PanelLeftClose, PanelLeftOpen, Plus } from "lucide-vue-next"
import { computed, ref } from "vue"

import type { ConnectionState } from "../../composables/useConnection"
import { useWorkspace } from "../../composables/workspaceContext"
import type { ConnectionKind } from "../../lib/connectionKind"
import { getConnectionKind, getConnectionKindLabel } from "../../lib/connectionKind"
import { normalizeConnectUri } from "../../lib/lancedbUri"
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
	isSavingProfile,
	addProfile,
	errorMessage,
	setStatus,
	setError,
	clearMessages,
} = useWorkspace()
const isCreateModalOpen = ref(false)

type ProfileFilterKind = "all" | ConnectionKind
const filterKind = ref<ProfileFilterKind>("all")

const expandedWidth = 320
const collapsedWidth = 72

const filteredProfiles = computed(() => {
	if (filterKind.value === "all") {
		return props.profiles
	}
	return props.profiles.filter((profile) => getConnectionKind(profile.uri) === filterKind.value)
})

const shouldVirtualize = computed(() => isCollapsed.value && filteredProfiles.value.length > 12)

const sidebarWidth = computed(() => (isCollapsed.value ? collapsedWidth : expandedWidth))

const createKind = computed<ConnectionKind>(() => {
	const value = profileForm.value.uri.trim()
	if (!value) {
		return "local"
	}
	return getConnectionKind(value)
})

const uriPlaceholder = computed(() => {
	switch (createKind.value) {
		case "local":
			return "例如：E:\\data\\sample-db（数据库目录）"
		case "s3":
			return "例如：s3://bucket/path"
		case "remote":
			return "例如：db://host:port"
		case "gcs":
			return "例如：gs://bucket/path"
		case "azure":
			return "例如：az://container/path"
		default:
			return "请输入 URI"
	}
})

const showLocalPicker = computed(() => createKind.value === "local")

async function pickLocalFolder() {
	clearMessages()
	try {
		const selected = await open({
			directory: true,
			multiple: false,
			title: "选择 LanceDB 数据库目录",
		})

		if (!selected || Array.isArray(selected)) {
			return
		}

		profileForm.value.uri = normalizeConnectUri(selected)
	} catch (error) {
		const message = error instanceof Error ? error.message : "打开文件夹选择器失败"
		setError(message)
	}
}

function toggleCollapse() {
	isCollapsed.value = !isCollapsed.value
}

function openCreateModal() {
	// Always open first; avoid any transient state issues preventing the modal from showing.
	isCreateModalOpen.value = true
	try {
		clearMessages()
		setStatus("已打开新建连接")
		const form = profileForm.value
		if (form && !form.name && !form.uri) {
			form.storageOptionsJson ||= "{}"
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : "打开新建连接弹窗失败"
		setError(message)
	}
}

function closeCreateModal() {
	isCreateModalOpen.value = false
}

async function saveProfile() {
	clearMessages()
	await addProfile()
	if (!errorMessage.value) {
		isCreateModalOpen.value = false
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
		class="flex h-full shrink-0 flex-col border-r border-slate-200 bg-white transition-[width] duration-200 ease-out"
		:style="{ width: `${sidebarWidth}px` }"
	>
		<div
			class="flex items-center px-4 pt-4"
			:class="isCollapsed ? 'justify-center' : 'justify-between'"
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
				<NButton size="small" quaternary @click="toggleCollapse">
					<PanelLeftClose v-if="!isCollapsed" class="h-4 w-4" />
					<PanelLeftOpen v-else class="h-4 w-4" />
				</NButton>
			</div>
		</div>

		<div class="mt-4 px-3" :class="isCollapsed ? 'px-2' : 'px-3'">
			<div class="flex items-center gap-2" :class="isCollapsed ? 'justify-center' : ''">
				<NPopover trigger="click" placement="bottom-start">
					<template #trigger>
						<NButton size="small" quaternary>
							<Filter class="h-4 w-4" />
							<span v-if="!isCollapsed" class="ml-2">筛选</span>
						</NButton>
					</template>
					<div class="w-56 space-y-2">
						<div class="text-xs font-medium text-slate-700">连接类型</div>
						<NRadioGroup v-model:value="filterKind" size="small">
							<div class="grid grid-cols-2 gap-2">
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

				<NButton size="small" type="primary" @click.stop="openCreateModal">
					<Plus class="h-4 w-4" />
					<span v-if="!isCollapsed" class="ml-2">新建</span>
				</NButton>
			</div>
		</div>

		<div class="mt-4 flex min-h-0 flex-1 flex-col overflow-hidden px-3 pb-4">
			<div class="min-h-0 flex-1 overflow-y-auto">
				<NEmpty v-if="!filteredProfiles.length" description="暂无连接档案" />
				<template v-else>
					<NVirtualList v-if="shouldVirtualize" :items="filteredProfiles" :item-size="92">
						<template #default="{ item }">
							<div class="pb-3">
								<ConnectionItem
									:key="item.id"
									:profile="item"
									:state="connectionStates[item.id]"
									:selected="item.id === activeProfileId"
									:collapsed="isCollapsed"
									@select="onSelectProfile(item.id)"
									@connect="selectAndConnect(item.id)"
									@refresh="selectAndRefresh(item.id)"
									@open-table="(name) => selectAndOpenTable(item.id, name)"
								/>
							</div>
						</template>
					</NVirtualList>
					<div v-else class="space-y-3">
						<ConnectionItem
							v-for="profile in filteredProfiles"
							:key="profile.id"
							:profile="profile"
							:state="connectionStates[profile.id]"
							:selected="profile.id === activeProfileId"
							:collapsed="isCollapsed"
							@select="onSelectProfile(profile.id)"
							@connect="selectAndConnect(profile.id)"
							@refresh="selectAndRefresh(profile.id)"
							@open-table="(name) => selectAndOpenTable(profile.id, name)"
						/>
					</div>
				</template>
			</div>
		</div>

		<NModal
			v-model:show="isCreateModalOpen"
			preset="card"
			title="新建连接"
			:to="'body'"
			:style="{ width: '520px' }"
		>
			<div class="space-y-3">
				<div class="space-y-1">
					<label class="text-xs text-slate-500">连接名称</label>
					<NInput v-model:value="profileForm.name" placeholder="例如：本地样例库" />
				</div>
				<div class="space-y-1">
					<label class="text-xs text-slate-500">URI</label>
					<div class="flex items-center gap-2">
						<NInput
							v-model:value="profileForm.uri"
							class="flex-1"
							:placeholder="uriPlaceholder"
						/>
						<NButton
							v-if="showLocalPicker"
							size="small"
							quaternary
							@click="pickLocalFolder"
						>
							<FolderOpen class="h-4 w-4" />
							<span class="ml-1">选择文件夹</span>
						</NButton>
					</div>
					<div v-if="showLocalPicker" class="text-xs text-slate-400">
						选择 LanceDB 的数据库根目录（例如 sample-db）。如果误选了 items.lance 这类 *.lance 目录，会自动改用它的上级目录。
					</div>
				</div>
				<div class="space-y-1">
					<label class="text-xs text-slate-500">storageOptions (JSON)</label>
					<NInput
						v-model:value="profileForm.storageOptionsJson"
						type="textarea"
						:autosize="{ minRows: 4, maxRows: 10 }"
						placeholder='{"aws_region": "us-east-1"}'
					/>
				</div>

				<div class="flex items-center justify-end gap-2 pt-2">
					<NButton size="small" quaternary :disabled="isSavingProfile" @click="closeCreateModal">
						取消
					</NButton>
					<NButton size="small" type="primary" :loading="isSavingProfile" @click="saveProfile">
						保存
					</NButton>
				</div>
			</div>
		</NModal>
	</aside>
</template>

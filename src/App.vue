<script setup lang="ts">
import { computed } from "vue"
import { RouterView, useRoute, useRouter } from "vue-router"

import StatusMessageBridge from "./components/StatusMessageBridge.vue"
import Sidebar from "./components/sidebar/Sidebar.vue"
import { useConnection } from "./composables/useConnection"
import { useProfiles } from "./composables/useProfiles"
import { useStatusMessages } from "./composables/useStatusMessages"
import { provideWorkspace } from "./composables/workspaceContext"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "./lib/connectionKind"
import { themeOverrides } from "./theme/naiveTheme"

type TagType = "default" | "info" | "warning" | "error" | "success" | "primary"
type WorkspaceName = "explorer" | "search"

const route = useRoute()
const router = useRouter()

const { statusMessage, errorMessage, setStatus, setError, clearMessages } = useStatusMessages()
const {
	profiles,
	activeProfileId,
	activeProfile,
	profileForm,
	isSavingProfile,
	addProfile,
	updateProfile,
	deleteProfile,
	setProfileLastConnected,
	selectProfile,
} = useProfiles({
	onStatus: setStatus,
	onError: setError,
})
const {
	connectionStates,
	activeConnection,
	connectionId,
	tables,
	activeTableName,
	activeTableId,
	schema,
	isConnecting,
	isRefreshing,
	isOpening,
	connectProfile,
	refreshTables,
	openTable,
	resetConnection,
} = useConnection(profiles, activeProfileId, {
	onStatus: setStatus,
	onError: setError,
	onConnected: setProfileLastConnected,
})

provideWorkspace({
	profiles,
	activeProfileId,
	activeProfile,
	profileForm,
	isSavingProfile,
	addProfile,
	updateProfile,
	deleteProfile,
	setProfileLastConnected,
	selectProfile,
	connectionStates,
	activeConnection,
	connectionId,
	tables,
	activeTableName,
	activeTableId,
	schema,
	isConnecting,
	isRefreshing,
	isOpening,
	connectProfile,
	refreshTables,
	openTable,
	resetConnection,
	statusMessage,
	errorMessage,
	setStatus,
	setError,
	clearMessages,
})

const activeConnectionSummary = computed<{
	label: string
	connected: boolean
	kindLabel: string
	kindTagType: TagType
}>(() => {
	if (!activeProfile.value) {
		return {
			label: "尚未选择连接",
			connected: false,
			kindLabel: "Unknown",
			kindTagType: "default" as const,
		}
	}
	const kind = getConnectionKind(activeProfile.value.uri)
	const connected = Boolean(connectionId.value)
	return {
		label: activeProfile.value.name,
		connected,
		kindLabel: getConnectionKindLabel(kind),
		kindTagType: getConnectionKindTagType(kind),
	}
})

const workspaceTab = computed<WorkspaceName>({
	get: () => (route.name === "search" ? "search" : "explorer"),
	set: (name) => {
		void router.push({ name })
	},
})

const isDialogRoute = computed(() => route.meta.layout === "dialog")
</script>

<template>
	<NConfigProvider :theme-overrides="themeOverrides">
		<NGlobalStyle />
		<NMessageProvider>
			<StatusMessageBridge
				:status-message="statusMessage"
				:error-message="errorMessage"
			/>
			<div class="h-screen w-screen overflow-hidden bg-slate-50">
				<div v-if="isDialogRoute" class="h-full w-full">
					<RouterView />
				</div>
				<div v-else class="flex h-full min-h-0">
					<Sidebar
						:profiles="profiles"
						:active-profile-id="activeProfileId"
						:connection-states="connectionStates"
						:on-select-profile="selectProfile"
						:on-connect-profile="connectProfile"
						:on-refresh-tables="refreshTables"
						:on-open-table="openTable"
					/>

					<main class="min-w-0 flex-1 overflow-y-auto">
						<div class="p-6">
							<div class="mx-auto w-full max-w-[1400px] space-y-4">
								<div
									class="sticky top-0 z-10 -mx-6 bg-slate-50/90 px-6 pb-4 pt-6 backdrop-blur"
								>
									<NCard size="small" title="连接状态" class="shadow-sm">
										<div class="space-y-2">
											<NAlert v-if="statusMessage" type="success" :bordered="false">
												{{ statusMessage }}
											</NAlert>
											<NAlert v-if="errorMessage" type="error" :bordered="false">
												{{ errorMessage }}
											</NAlert>
											<div class="flex flex-wrap items-center gap-2 text-sm text-slate-600">
												<div class="flex items-center gap-2">
													<span
														class="h-2 w-2 rounded-full"
														:class="
															activeConnectionSummary.connected
																? 'bg-emerald-500'
																: 'bg-slate-300'
														"
													/>
													<span>{{ activeConnectionSummary.label }}</span>
												</div>
												<NTag size="small" :type="activeConnectionSummary.kindTagType">
													{{ activeConnectionSummary.kindLabel }}
												</NTag>
												<span class="text-slate-400">•</span>
												<span>
													{{ connectionId ? `表数量：${tables.length}` : "尚未连接数据库" }}
												</span>
											</div>
									</div>
								</NCard>

								<div class="mt-3 flex items-center">
									<NRadioGroup v-model:value="workspaceTab" size="small">
										<NRadioButton value="explorer">资源浏览</NRadioButton>
										<NRadioButton value="search">检索工作台</NRadioButton>
									</NRadioGroup>
								</div>
							</div>

							<RouterView />
						</div>
					</div>
				</main>
			</div>
		</div>
		</NMessageProvider>
	</NConfigProvider>
</template>
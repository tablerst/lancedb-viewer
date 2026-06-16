<script setup lang="ts">
import { darkTheme, dateZhCN, zhCN } from "naive-ui"
import { computed, watch, watchEffect } from "vue"
import { RouterView, useRoute } from "vue-router"
import PrimaryNav from "./components/layout/PrimaryNav.vue"
import StatusMessageBridge from "./components/StatusMessageBridge.vue"
import Sidebar from "./components/sidebar/Sidebar.vue"
import { shouldClearTransientMessagesOnRouteChange } from "./composables/statusMessagePolicy"
import { useConnection } from "./composables/useConnection"
import { useProfiles } from "./composables/useProfiles"
import { useStatusMessages } from "./composables/useStatusMessages"
import { useTheme } from "./composables/useTheme"
import { provideWorkspace } from "./composables/workspaceContext"
import { darkThemeOverrides, themeOverrides } from "./theme/naiveTheme"

const { isDark } = useTheme()

const route = useRoute()

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
	isDisconnecting,
	connectProfile,
	disconnectProfile,
	refreshTables,
	openTable,
	refreshSchema,
	resetConnection,
	clearActiveTable,
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
	isDisconnecting,
	connectProfile,
	disconnectProfile,
	refreshTables,
	openTable,
	refreshSchema,
	resetConnection,
	clearActiveTable,
	statusMessage,
	errorMessage,
	setStatus,
	setError,
	clearMessages,
})

const isDialogRoute = computed(() => route.meta.layout === "dialog")
const isFullWidthRoute = computed(() => {
	const name = route.name
	return (
		name === "explorer" ||
		name === "connection-explorer" ||
		name === "table-tab" ||
		name === "search" ||
		name === "connection-search"
	)
})

/** Select profile and auto-connect if not yet connected. */
async function selectAndAutoConnect(profileId: string) {
	await selectProfile(profileId)
	const state = connectionStates.value[profileId]
	if (
		state &&
		!state.connectionId.value &&
		!state.isConnecting.value &&
		!state.isDisconnecting.value
	) {
		await connectProfile(profileId)
	}
}

watchEffect(() => {
	const raw = route.params.id
	if (typeof raw !== "string") {
		return
	}
	if (!profiles.value.length) {
		return
	}
	if (activeProfileId.value === raw) {
		return
	}
	const exists = profiles.value.some((profile) => profile.id === raw)
	if (!exists) {
		return
	}
	void selectProfile(raw)
})

watch(
	() => route.fullPath,
	(_value, previous) => {
		if (shouldClearTransientMessagesOnRouteChange(previous)) {
			clearMessages()
		}
	}
)
</script>

<template>
	<NConfigProvider
		:theme="isDark ? darkTheme : undefined"
		:theme-overrides="isDark ? darkThemeOverrides : themeOverrides"
		:locale="zhCN"
		:date-locale="dateZhCN"
	>
		<NGlobalStyle />
		<NMessageProvider>
			<NDialogProvider>
				<StatusMessageBridge :status-message="statusMessage" :error-message="errorMessage" />
				<div class="h-screen w-screen overflow-hidden bg-slate-50">
					<div v-if="isDialogRoute" class="h-full w-full">
						<RouterView />
					</div>
					<div v-else class="flex h-full min-h-0">
						<PrimaryNav />
						<Sidebar
							class="hidden md:flex"
							:profiles="profiles"
							:active-profile-id="activeProfileId"
							:connection-states="connectionStates"
							:on-select-profile="selectAndAutoConnect"
							:on-connect-profile="connectProfile"
							:on-disconnect-profile="disconnectProfile"
							:on-refresh-tables="refreshTables"
							:on-open-table="openTable"
						/>

						<main class="min-w-0 flex-1 flex flex-col overflow-hidden">
							<div v-if="errorMessage" class="px-6 pt-4">
								<NAlert
									type="error"
									closable
									@close="clearMessages"
								>
									{{ errorMessage }}
								</NAlert>
							</div>
							<div class="min-h-0 flex-1 overflow-y-auto p-6">
								<div :class="isFullWidthRoute ? 'w-full h-full' : 'mx-auto w-full max-w-[1600px]'">
									<RouterView />
								</div>
							</div>
						</main>
					</div>
				</div>
			</NDialogProvider>
		</NMessageProvider>
	</NConfigProvider>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { RouterView, useRoute } from "vue-router"

import StatusMessageBridge from "./components/StatusMessageBridge.vue"
import Sidebar from "./components/sidebar/Sidebar.vue"
import { useConnection } from "./composables/useConnection"
import { useProfiles } from "./composables/useProfiles"
import { useStatusMessages } from "./composables/useStatusMessages"
import { provideWorkspace } from "./composables/workspaceContext"
import { themeOverrides } from "./theme/naiveTheme"

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
	connectProfile,
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
	connectProfile,
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
</script>

<template>
	<NConfigProvider :theme-overrides="themeOverrides">
		<NGlobalStyle />
		<NMessageProvider>
			<StatusMessageBridge :status-message="statusMessage" :error-message="errorMessage" />
			<div class="h-screen w-screen overflow-hidden bg-slate-50">
				<div v-if="isDialogRoute" class="h-full w-full">
					<RouterView />
				</div>
				<div v-else class="flex h-full min-h-0">
					<Sidebar :profiles="profiles" :active-profile-id="activeProfileId"
						:connection-states="connectionStates" :on-select-profile="selectProfile"
						:on-connect-profile="connectProfile" :on-refresh-tables="refreshTables"
						:on-open-table="openTable" />

					<main class="min-w-0 flex-1 overflow-y-auto">
						<div class="p-6">
							<div class="mx-auto w-full max-w-[1400px]">
								<RouterView />
							</div>
						</div>
					</main>
				</div>
			</div>
		</NMessageProvider>
	</NConfigProvider>
</template>
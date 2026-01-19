<script setup lang="ts">
import { watch } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";

import { useConnection } from "./composables/useConnection";
import { useProfiles } from "./composables/useProfiles";
import { useStatusMessages } from "./composables/useStatusMessages";
import { provideWorkspace } from "./composables/workspaceContext";
import { themeOverrides } from "./theme/naiveTheme";

const { statusMessage, errorMessage, setStatus, setError, clearMessages } = useStatusMessages();
const {
	profiles,
	activeProfileId,
	activeProfile,
	profileForm,
	isSavingProfile,
	addProfile,
	selectProfile,
} = useProfiles({
	onStatus: setStatus,
	onError: setError,
});
const {
	connectionId,
	tables,
	activeTableName,
	activeTableId,
	schema,
	isConnecting,
	isRefreshing,
	isOpening,
	connectActiveProfile,
	refreshTables,
	openTable,
	resetConnection,
} = useConnection(activeProfile, {
	onStatus: setStatus,
	onError: setError,
});

watch(activeProfileId, () => {
	resetConnection();
	clearMessages();
});

provideWorkspace({
	profiles,
	activeProfileId,
	activeProfile,
	profileForm,
	isSavingProfile,
	addProfile,
	selectProfile,
	connectionId,
	tables,
	activeTableName,
	activeTableId,
	schema,
	isConnecting,
	isRefreshing,
	isOpening,
	connectActiveProfile,
	refreshTables,
	openTable,
	resetConnection,
	statusMessage,
	errorMessage,
	setStatus,
	setError,
	clearMessages,
});

const route = useRoute();
const navigationItems = [
	{ label: "资源浏览", to: "/" },
	{ label: "检索工作台", to: "/search" },
];

const isActiveRoute = (path: string) => route.path === path;
</script>

<template>
	<NConfigProvider :theme-overrides="themeOverrides">
		<NGlobalStyle />
		<NLayout class="min-h-screen bg-slate-50">
			<NLayoutHeader bordered class="px-6 py-4">
				<div class="flex items-center justify-between">
					<div>
						<div class="text-lg font-semibold text-slate-900">LanceDB Studio</div>
						<p class="text-xs text-slate-500">JSON-first IPC · 体验闭环优先</p>
					</div>
					<div class="flex items-center gap-2">
						<NTag type="info" size="small">v1</NTag>
						<NTag v-if="connectionId" type="success" size="small">已连接</NTag>
					</div>
				</div>
			</NLayoutHeader>

			<NLayout has-sider>
				<NLayoutSider bordered width="320" content-style="padding: 16px;">
					<div class="space-y-4">
						<NCard size="small" title="导航" class="shadow-sm">
							<div class="grid gap-2">
								<NButton
									v-for="item in navigationItems"
									:key="item.to"
									block
									:tag="RouterLink"
									:to="item.to"
									:type="isActiveRoute(item.to) ? 'primary' : 'default'"
									:secondary="!isActiveRoute(item.to)"
								>
									{{ item.label }}
								</NButton>
							</div>
						</NCard>

						<NCard size="small" title="连接档案" class="shadow-sm">
							<NEmpty v-if="!profiles.length" description="暂无连接档案" />
							<div v-else class="space-y-2">
								<NButton
									v-for="profile in profiles"
									:key="profile.id"
									block
									:type="profile.id === activeProfileId ? 'primary' : 'default'"
									:secondary="profile.id !== activeProfileId"
									@click="selectProfile(profile.id)"
								>
									<div class="flex flex-col text-left">
										<span class="font-medium">{{ profile.name }}</span>
										<span class="text-xs text-slate-500">{{ profile.uri }}</span>
									</div>
								</NButton>
							</div>
							<div class="mt-3">
								<NButton
									block
									type="primary"
									:loading="isConnecting"
									:disabled="!activeProfile"
									@click="connectActiveProfile"
								>
									连接当前档案
								</NButton>
							</div>
						</NCard>

						<NCard size="small" title="新增档案" class="shadow-sm">
							<NForm :model="profileForm" size="small">
								<NFormItem label="名称">
									<NInput v-model:value="profileForm.name" placeholder="本地 LanceDB" />
								</NFormItem>
								<NFormItem label="URI">
									<NInput
										v-model:value="profileForm.uri"
										placeholder="/path/to/db 或 s3://bucket/db"
									/>
								</NFormItem>
								<NFormItem label="storageOptions (JSON)">
									<NInput
										v-model:value="profileForm.storageOptionsJson"
										type="textarea"
										:autosize="{ minRows: 3, maxRows: 6 }"
									/>
								</NFormItem>
							</NForm>
							<NButton
								block
								secondary
								type="primary"
								:loading="isSavingProfile"
								@click="addProfile"
							>
								保存档案
							</NButton>
						</NCard>

						<NCard size="small" title="表列表" class="shadow-sm">
							<template #header-extra>
								<NButton
									size="tiny"
									quaternary
									:loading="isRefreshing"
									:disabled="!connectionId"
									@click="refreshTables"
								>
									刷新
								</NButton>
							</template>
							<NEmpty v-if="!tables.length" description="暂无数据表" />
							<div v-else class="space-y-2">
								<NButton
									v-for="table in tables"
									:key="table.name"
									block
									:loading="isOpening && table.name === activeTableName"
									:type="table.name === activeTableName ? 'primary' : 'default'"
									:secondary="table.name !== activeTableName"
									@click="openTable(table.name)"
								>
									{{ table.name }}
								</NButton>
							</div>
						</NCard>
					</div>
				</NLayoutSider>

				<NLayoutContent class="p-6">
					<div class="grid gap-4">
						<NCard size="small" title="连接状态" class="shadow-sm">
							<div class="space-y-2">
								<NAlert v-if="statusMessage" type="success" :bordered="false">
									{{ statusMessage }}
								</NAlert>
								<NAlert v-if="errorMessage" type="error" :bordered="false">
									{{ errorMessage }}
								</NAlert>
								<p v-if="!connectionId" class="text-sm text-slate-500">
									尚未连接数据库
								</p>
								<div v-else class="flex flex-wrap gap-2 text-sm text-slate-600">
									<span>当前连接：{{ activeProfile?.name ?? "未知" }}</span>
									<span class="text-slate-400">•</span>
									<span>表数量：{{ tables.length }}</span>
								</div>
							</div>
						</NCard>

						<RouterView />
					</div>
				</NLayoutContent>
			</NLayout>
		</NLayout>
	</NConfigProvider>
</template>
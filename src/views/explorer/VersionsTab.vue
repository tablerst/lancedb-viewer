<script setup lang="ts">
import { GitBranch, History, RefreshCw, RotateCcw } from "lucide-vue-next"
import { computed, inject, ref, watch } from "vue"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { VersionInfoV1 } from "../../ipc/v1"
import { formatTimestamp } from "../../lib/formatters"
import {
	checkoutTableLatestV1,
	checkoutTableVersionV1,
	cloneTableV1,
	getTableVersionV1,
	listVersionsV1,
	unwrapEnvelope,
} from "../../lib/tauriClient"
import { getMetadataEntries, TRIGGER_DATA_REFRESH_KEY } from "./explorerShared"

const {
	activeProfileId,
	connectionId,
	activeTableId,
	setError,
	setStatus,
	refreshSchema,
	refreshTables,
} = useWorkspace()

const triggerDataRefresh = inject(TRIGGER_DATA_REFRESH_KEY, () => {})

const hasActiveTable = computed(() => Boolean(activeTableId.value))

// ── Versions ───────────────────────────────────────────

const versions = ref<VersionInfoV1[]>([])
const isLoadingVersions = ref(false)
const versionError = ref("")
const currentVersion = ref<number | null>(null)
const checkoutVersion = ref<number | null>(null)
const { execute: execCheckoutVersion, isLoading: isCheckingOutVersion } = useCommand("切换版本失败")
const { execute: execCheckoutLatest, isLoading: isCheckingOutLatest } =
	useCommand("恢复最新版本失败")

const timelineItems = computed(() =>
	versions.value.map((v) => {
		const isCurrent = v.version === currentVersion.value
		return {
			version: v.version,
			isCurrent,
			type: (isCurrent ? "success" : "default") as "success" | "default",
			title: `v${v.version}${isCurrent ? "  (当前)" : ""}`,
			time: formatTimestamp(v.timestamp),
			metadataEntries: getMetadataEntries(v.metadata),
		}
	})
)

async function loadVersions() {
	const tableId = activeTableId.value
	if (!tableId || isLoadingVersions.value) {
		return
	}
	try {
		isLoadingVersions.value = true
		versionError.value = ""
		const response = unwrapEnvelope(await listVersionsV1({ tableId }))
		versions.value = response.versions
	} catch (error) {
		const msg = error instanceof Error ? error.message : "获取版本列表失败"
		versionError.value = msg
		setError(msg)
	} finally {
		isLoadingVersions.value = false
	}
}

async function loadCurrentVersion() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}
	try {
		const response = unwrapEnvelope(await getTableVersionV1({ tableId }))
		currentVersion.value = response.version
	} catch (error) {
		setError(error instanceof Error ? error.message : "获取当前版本失败")
	}
}

async function submitCheckoutVersion() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const version = checkoutVersion.value
	if (!profileId || !tableId) {
		return
	}
	if (version === null) {
		setError("请输入版本号")
		return
	}
	if (version < 0) {
		setError("版本号不能为负数")
		return
	}
	await execCheckoutVersion(async () => {
		const response = unwrapEnvelope(await checkoutTableVersionV1({ tableId, version }))
		currentVersion.value = response.version
		setStatus(`已切换到版本 ${response.version}`)
		await refreshSchema(profileId)
		triggerDataRefresh()
	})
}

async function submitCheckoutLatest() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId) {
		return
	}
	await execCheckoutLatest(async () => {
		const response = unwrapEnvelope(await checkoutTableLatestV1({ tableId }))
		currentVersion.value = response.version
		setStatus(`已回到最新版本 ${response.version}`)
		await refreshSchema(profileId)
		triggerDataRefresh()
	})
}

// ── Clone ──────────────────────────────────────────────

const cloneTargetName = ref("")
const cloneSourceVersion = ref<number | null>(null)
const cloneIsShallow = ref(true)
const { execute: execCloneTable, isLoading: isCloningTable } = useCommand("克隆表失败")

async function submitCloneTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableId = activeTableId.value
	if (!profileId || !currentConnectionId || !tableId) {
		return
	}
	const targetName = cloneTargetName.value.trim()
	if (!targetName) {
		setError("请输入克隆表名")
		return
	}
	await execCloneTable(async () => {
		const response = unwrapEnvelope(
			await cloneTableV1({
				connectionId: currentConnectionId,
				tableId,
				targetTableName: targetName,
				sourceVersion: cloneSourceVersion.value ?? undefined,
				isShallow: cloneIsShallow.value,
			})
		)
		setStatus(`已创建克隆表 ${response.name}`)
		cloneTargetName.value = ""
		cloneSourceVersion.value = null
		cloneIsShallow.value = true
		await refreshTables(profileId)
	})
}

// ── Watchers ───────────────────────────────────────────

watch(
	activeTableId,
	() => {
		versions.value = []
		versionError.value = ""
		currentVersion.value = null
		checkoutVersion.value = null
		isLoadingVersions.value = false
		isCheckingOutVersion.value = false
		isCheckingOutLatest.value = false
		cloneTargetName.value = ""
		cloneSourceVersion.value = null
		cloneIsShallow.value = true
		isCloningTable.value = false
		if (activeTableId.value) {
			void loadVersions()
			void loadCurrentVersion()
		}
	},
	{ immediate: true }
)
</script>

<template>
	<div class="versions-workbench">
		<section class="versions-main-panel">
			<header class="versions-header">
				<div>
					<h2 class="versions-title">版本</h2>
					<p class="versions-subtitle">
						当前版本
						<span class="current-version">{{ currentVersion ?? "—" }}</span>
					</p>
				</div>
				<div class="versions-actions">
					<NButton
						secondary
						size="small"
						:loading="isLoadingVersions"
						:disabled="!hasActiveTable"
						@click="loadVersions"
					>
						<template #icon>
							<RefreshCw class="h-4 w-4" />
						</template>
						刷新列表
					</NButton>
					<NButton
						quaternary
						size="small"
						:disabled="!hasActiveTable"
						@click="loadCurrentVersion"
					>
						当前版本
					</NButton>
				</div>
			</header>

			<NAlert v-if="versionError" type="error" :bordered="false" class="versions-alert">
				{{ versionError }}
			</NAlert>

			<div v-if="isLoadingVersions && !versions.length" class="versions-skeleton">
				<NSkeleton text :repeat="4" class="w-full" />
			</div>
			<div v-else-if="timelineItems.length" class="versions-timeline">
				<article
					v-for="item in timelineItems"
					:key="item.version"
					class="version-row"
					:class="{ 'version-row--current': item.isCurrent }"
				>
					<div class="version-marker" aria-hidden="true" />
					<div class="version-body">
						<div class="version-row-header">
							<div class="version-id">
								<span>v{{ item.version }}</span>
								<NTag
									v-if="item.isCurrent"
									size="small"
									type="success"
									:bordered="false"
								>
									当前
								</NTag>
							</div>
							<time class="version-time">{{ item.time }}</time>
						</div>
						<div v-if="item.metadataEntries.length" class="version-metadata-grid">
							<div
								v-for="entry in item.metadataEntries"
								:key="`${item.version}-${entry.key}`"
								class="version-metadata-entry"
							>
								<span class="version-metadata-key">{{ entry.key }}</span>
								<span class="version-metadata-value">{{ entry.value }}</span>
							</div>
						</div>
						<div v-else class="version-empty-metadata">无 metadata</div>
					</div>
				</article>
			</div>
			<NEmpty v-else description="暂无版本记录" class="versions-empty" />
		</section>

		<aside class="versions-command-panel">
			<section class="version-command-section">
				<div class="version-command-heading">
					<div>
						<h3 class="version-command-title">打开版本</h3>
						<p class="version-command-subtitle">切换当前表到指定历史版本</p>
					</div>
					<History class="version-command-icon" />
				</div>
				<label class="version-field" for="checkout-version-input">
					<span>版本号</span>
					<NInputNumber
						v-model:value="checkoutVersion"
						:min="0"
						:show-button="false"
						:input-props="{
							id: 'checkout-version-input',
							'aria-label': '要打开的版本号',
						}"
						:disabled="!hasActiveTable"
					/>
				</label>
				<div class="version-command-actions">
					<NButton
						type="primary"
						:loading="isCheckingOutVersion"
						:disabled="!hasActiveTable"
						@click="submitCheckoutVersion"
					>
						打开版本
					</NButton>
					<NButton
						secondary
						:loading="isCheckingOutLatest"
						:disabled="!hasActiveTable"
						@click="submitCheckoutLatest"
					>
						<template #icon>
							<RotateCcw class="h-4 w-4" />
						</template>
						回到最新
					</NButton>
				</div>
			</section>

			<section class="version-command-section">
				<div class="version-command-heading">
					<div>
						<h3 class="version-command-title">克隆/分支</h3>
						<p class="version-command-subtitle">从当前表或指定版本派生新表</p>
					</div>
					<GitBranch class="version-command-icon" />
				</div>
				<div class="version-field-grid">
					<label class="version-field">
						<span>新表名</span>
						<NInput
							v-model:value="cloneTargetName"
							placeholder="clone_table"
							:disabled="!hasActiveTable"
						/>
					</label>
					<label class="version-field" for="clone-source-version-input">
						<span>源版本</span>
						<NInputNumber
							v-model:value="cloneSourceVersion"
							:min="0"
							placeholder="留空使用最新"
							:show-button="false"
							:input-props="{
								id: 'clone-source-version-input',
								'aria-label': '克隆源版本',
							}"
							:disabled="!hasActiveTable"
						/>
					</label>
				</div>
				<div class="clone-mode-row">
					<NCheckbox v-model:checked="cloneIsShallow" :disabled="!hasActiveTable">
						浅克隆
					</NCheckbox>
					<span>共享数据文件，适合分支试验；深克隆暂未实现。</span>
				</div>
				<div class="version-command-actions version-command-actions--end">
					<NButton
						type="primary"
						:loading="isCloningTable"
						:disabled="!hasActiveTable"
						@click="submitCloneTable"
					>
						创建克隆
					</NButton>
				</div>
			</section>
		</aside>
	</div>
</template>

<style scoped>
.versions-workbench {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(320px, 380px);
	gap: 16px;
	align-items: start;
}

.versions-main-panel,
.versions-command-panel {
	min-width: 0;
}

.versions-main-panel,
.version-command-section {
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
}

.versions-main-panel {
	overflow: hidden;
}

.versions-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	padding: 16px 18px 12px;
	border-bottom: 1px solid var(--app-rule);
}

.versions-title,
.version-command-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 15px;
	font-weight: 680;
	line-height: 1.3;
}

.versions-subtitle,
.version-command-subtitle {
	margin: 4px 0 0;
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.4;
}

.current-version {
	margin-left: 6px;
	color: var(--app-ink);
	font-family: var(--app-mono-font);
	font-weight: 650;
}

.versions-actions {
	display: flex;
	flex-wrap: wrap;
	justify-content: flex-end;
	gap: 8px;
}

.versions-alert {
	margin: 12px 16px 0;
}

.versions-skeleton {
	padding: 18px;
}

.versions-timeline {
	position: relative;
	max-height: min(520px, calc(100vh - 270px));
	overflow-y: auto;
	padding: 16px 18px 18px 32px;
}

.versions-timeline::before {
	position: absolute;
	top: 18px;
	bottom: 18px;
	left: 20px;
	width: 1px;
	background: var(--app-rule-strong);
	content: "";
}

.version-row {
	position: relative;
	display: grid;
	grid-template-columns: 14px minmax(0, 1fr);
	gap: 12px;
	padding-bottom: 18px;
}

.version-row:last-child {
	padding-bottom: 0;
}

.version-marker {
	z-index: 1;
	width: 10px;
	height: 10px;
	margin-top: 7px;
	border: 2px solid var(--app-rule-strong);
	border-radius: 999px;
	background: var(--app-surface-elevated);
}

.version-row--current .version-marker {
	border-color: var(--app-success);
	box-shadow: 0 0 0 3px var(--app-success-soft);
}

.version-body {
	min-width: 0;
	padding: 10px 12px;
	border: 1px solid transparent;
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-surface-panel-muted) 54%, transparent);
}

.version-row--current .version-body {
	border-color: color-mix(in srgb, var(--app-success) 28%, var(--app-rule));
	background: color-mix(in srgb, var(--app-success-soft) 46%, var(--app-surface-elevated));
}

.version-row-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 12px;
	min-width: 0;
}

.version-id {
	display: flex;
	min-width: 0;
	align-items: center;
	gap: 8px;
	color: var(--app-ink-strong);
	font-family: var(--app-mono-font);
	font-size: 13px;
	font-weight: 680;
}

.version-time {
	flex: 0 0 auto;
	color: var(--app-muted);
	font-size: 12px;
}

.version-metadata-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
	gap: 5px 14px;
	margin-top: 10px;
	padding-top: 8px;
	border-top: 1px solid color-mix(in srgb, var(--app-rule) 70%, transparent);
}

.version-metadata-entry {
	display: flex;
	min-width: 0;
	align-items: center;
	gap: 6px;
	padding: 0;
	font-size: 12px;
	line-height: 1.5;
}

.version-metadata-key {
	flex: 0 0 auto;
	max-width: 45%;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-muted);
	font-weight: 600;
}

.version-metadata-key::after {
	content: ":";
}

.version-metadata-value {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-ink);
	font-family: var(--app-mono-font);
	font-size: 11px;
}

.version-empty-metadata {
	margin-top: 8px;
	color: var(--app-subtle);
	font-size: 12px;
}

.versions-empty {
	padding: 36px 0 42px;
}

.versions-command-panel {
	display: grid;
	gap: 12px;
}

.version-command-section {
	padding: 14px;
}

.version-command-heading {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	margin-bottom: 14px;
}

.version-command-icon {
	width: 18px;
	height: 18px;
	color: var(--app-subtle);
}

.version-field-grid {
	display: grid;
	gap: 10px;
}

.version-field {
	display: grid;
	min-width: 0;
	gap: 5px;
	color: var(--app-muted);
	font-size: 12px;
	font-weight: 620;
	line-height: 1.2;
}

.version-field :deep(.n-input),
.version-field :deep(.n-input-number) {
	width: 100%;
}

.version-command-actions {
	display: flex;
	flex-wrap: wrap;
	align-items: center;
	justify-content: space-between;
	gap: 10px;
	margin-top: 14px;
}

.version-command-actions--end {
	justify-content: flex-end;
}

.clone-mode-row {
	display: flex;
	align-items: flex-start;
	gap: 10px;
	margin-top: 12px;
	padding-top: 12px;
	border-top: 1px solid var(--app-rule);
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.45;
}

@media (max-width: 1100px) {
	.versions-workbench {
		grid-template-columns: 1fr;
	}

	.versions-command-panel {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}

@media (max-width: 720px) {
	.versions-header,
	.version-row-header,
	.version-command-actions,
	.clone-mode-row {
		flex-direction: column;
		align-items: stretch;
	}

	.versions-command-panel {
		grid-template-columns: 1fr;
	}

	.versions-timeline {
		max-height: none;
	}
}
</style>

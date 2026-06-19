<script setup lang="ts">
import { GitBranch, RefreshCw, RotateCcw } from "lucide-vue-next"
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
const branchSourceVersion = ref<number | null>(null)
const { execute: execCheckoutVersion, isLoading: isCheckingOutVersion } = useCommand("切换版本失败")
const { execute: execCheckoutLatest, isLoading: isCheckingOutLatest } =
	useCommand("恢复最新版本失败")

const versionMetricPriority = [
	"total_rows",
	"total_fragments",
	"total_data_files",
	"total_files_size",
]

const versionMetricLabels: Record<string, string> = {
	total_rows: "Rows",
	total_fragments: "Fragments",
	total_data_files: "Files",
	total_files_size: "Size",
	total_data_file_rows: "Data rows",
	total_deletion_files: "Deletes",
	total_deletion_file_rows: "Deleted rows",
}

function formatMetricValue(key: string, value: string) {
	if (key.includes("size")) {
		return formatByteCount(value)
	}
	const numeric = Number(value)
	if (Number.isFinite(numeric) && value.trim() !== "") {
		return new Intl.NumberFormat("en-US").format(numeric)
	}
	return value
}

function formatByteCount(rawValue: string) {
	const bytes = Number(rawValue)
	if (!Number.isFinite(bytes) || bytes < 0) {
		return rawValue
	}
	if (bytes < 1024) {
		return `${bytes} B`
	}
	const units = ["KB", "MB", "GB", "TB"]
	let value = bytes / 1024
	let unitIndex = 0
	while (value >= 1024 && unitIndex < units.length - 1) {
		value /= 1024
		unitIndex += 1
	}
	const precision = value >= 10 ? 1 : 2
	return `${Number(value.toFixed(precision))} ${units[unitIndex]}`
}

function buildVersionMetrics(entries: ReturnType<typeof getMetadataEntries>) {
	const byKey = new Map(entries.map((entry) => [entry.key, entry]))
	const selectedKeys: string[] = []
	for (const key of versionMetricPriority) {
		if (byKey.has(key)) {
			selectedKeys.push(key)
		}
	}
	for (const entry of entries) {
		if (selectedKeys.length >= 4) {
			break
		}
		if (!selectedKeys.includes(entry.key)) {
			selectedKeys.push(entry.key)
		}
	}
	return selectedKeys.map((key) => {
		const entry = byKey.get(key)
		return {
			key,
			label: versionMetricLabels[key] ?? key,
			value: formatMetricValue(key, entry?.value ?? ""),
		}
	})
}

const timelineItems = computed(() =>
	[...versions.value]
		.sort((a, b) => {
			const timeDiff = new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
			return timeDiff === 0 ? b.version - a.version : timeDiff
		})
		.map((v) => {
			const isCurrent = v.version === currentVersion.value
			const isBranching = v.version === branchSourceVersion.value
			const metadataEntries = getMetadataEntries(v.metadata)
			const summaryEntries = buildVersionMetrics(metadataEntries)
			return {
				version: v.version,
				isCurrent,
				isBranching,
				time: formatTimestamp(v.timestamp),
				metadataEntries,
				moreMetadataCount: Math.max(0, metadataEntries.length - summaryEntries.length),
				summaryEntries,
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

async function submitCheckoutVersion(versionOverride?: number) {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const version = versionOverride ?? checkoutVersion.value
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
		checkoutVersion.value = null
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

function toggleBranchForm(version: number) {
	if (branchSourceVersion.value === version) {
		branchSourceVersion.value = null
		cloneSourceVersion.value = null
		return
	}
	branchSourceVersion.value = version
	cloneSourceVersion.value = version
}

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
		branchSourceVersion.value = null
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
		branchSourceVersion.value = null
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
		<section class="versions-graph-panel">
			<header class="versions-header">
				<div>
					<h2 class="versions-title">版本</h2>
					<p class="versions-subtitle">
						按时间倒序排列 · 当前版本
						<span class="current-version">{{ currentVersion ?? "—" }}</span>
					</p>
				</div>
				<div class="versions-actions">
					<div class="version-jump">
						<NInputNumber
							v-model:value="checkoutVersion"
							size="small"
							:min="0"
							:show-button="false"
							placeholder="版本号"
							:input-props="{
								'aria-label': '要打开的版本号',
							}"
							:disabled="!hasActiveTable"
						/>
						<NButton
							type="primary"
							secondary
							size="small"
							:loading="isCheckingOutVersion && checkoutVersion !== null"
							:disabled="!hasActiveTable || checkoutVersion === null"
							@click="submitCheckoutVersion()"
						>
							打开
						</NButton>
					</div>
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
						secondary
						size="small"
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
			</header>

			<NAlert v-if="versionError" type="error" :bordered="false" class="versions-alert">
				{{ versionError }}
			</NAlert>

			<div v-if="isLoadingVersions && !versions.length" class="versions-skeleton">
				<NSkeleton text :repeat="4" class="w-full" />
			</div>
			<div v-else-if="timelineItems.length" class="version-graph">
				<article
					v-for="item in timelineItems"
					:key="item.version"
					class="version-node"
					:class="{
						'version-node--current': item.isCurrent,
						'version-node--branching': item.isBranching,
					}"
				>
					<div class="graph-rail" aria-hidden="true">
						<span class="graph-dot" />
					</div>
					<div class="version-node-content">
						<div class="version-card">
							<div class="version-card-main">
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
							</div>

							<div v-if="item.summaryEntries.length" class="version-metrics-strip">
								<div
									v-for="entry in item.summaryEntries"
									:key="`${item.version}-${entry.key}`"
									class="version-metric"
									:title="`${entry.label}: ${entry.value}`"
								>
									<span class="version-metric-label">{{ entry.label }}</span>
									<span class="version-metric-value">{{ entry.value }}</span>
								</div>
								<span v-if="item.moreMetadataCount" class="version-more-metadata">
									+{{ item.moreMetadataCount }}
								</span>
							</div>
							<div v-else class="version-empty-metadata">无 metadata</div>

							<div class="version-row-actions">
								<NButton
									size="tiny"
									secondary
									:loading="isCheckingOutVersion && checkoutVersion === item.version"
									:disabled="!hasActiveTable || item.isCurrent"
									@click="
										checkoutVersion = item.version;
										submitCheckoutVersion(item.version)
									"
								>
									打开
								</NButton>
								<NButton
									size="tiny"
									:type="item.isBranching ? 'primary' : 'default'"
									secondary
									:disabled="!hasActiveTable"
									@click="toggleBranchForm(item.version)"
								>
									<template #icon>
										<GitBranch class="h-3.5 w-3.5" />
									</template>
									分支
								</NButton>
							</div>
						</div>

						<div v-if="item.isBranching" class="branch-editor">
							<div class="branch-source">
								从 <span>v{{ item.version }}</span> 派生新表
							</div>
							<div class="branch-grid">
								<label class="version-field">
									<span>新表名</span>
									<NInput
										v-model:value="cloneTargetName"
										size="small"
										placeholder="clone_table"
										:disabled="!hasActiveTable"
									/>
								</label>
								<div class="clone-mode-row">
									<NCheckbox
										v-model:checked="cloneIsShallow"
										size="small"
										:disabled="!hasActiveTable"
									>
										浅克隆
									</NCheckbox>
									<span>共享数据文件，适合分支试验；深克隆暂未实现。</span>
								</div>
								<div class="branch-actions">
									<NButton
										size="small"
										secondary
										:disabled="isCloningTable"
										@click="toggleBranchForm(item.version)"
									>
										取消
									</NButton>
									<NButton
										type="primary"
										size="small"
										:loading="isCloningTable"
										:disabled="!hasActiveTable"
										@click="submitCloneTable"
									>
										创建分支
									</NButton>
								</div>
							</div>
						</div>
					</div>
				</article>
			</div>
			<NEmpty v-else description="暂无版本记录" class="versions-empty" />
		</section>
	</div>
</template>

<style scoped>
.versions-workbench {
	display: block;
	min-width: 0;
}

.versions-graph-panel {
	min-width: 0;
	overflow: hidden;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
}

.versions-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	padding: 16px 18px 12px;
	border-bottom: 1px solid var(--app-rule);
}

.versions-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 15px;
	font-weight: 680;
	line-height: 1.3;
}

.versions-subtitle {
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

.version-jump {
	display: flex;
	align-items: center;
	gap: 6px;
}

.version-jump :deep(.n-input-number) {
	width: 112px;
}

.versions-alert {
	margin: 12px 16px 0;
}

.versions-skeleton {
	padding: 18px;
}

.version-graph {
	position: relative;
	max-height: min(560px, calc(100vh - 250px));
	overflow-x: hidden;
	overflow-y: auto;
	padding: 12px 14px 14px;
}

.version-node {
	position: relative;
	display: grid;
	grid-template-columns: 34px minmax(0, 1fr);
	gap: 12px;
	min-width: 0;
}

.version-node:not(:last-child) {
	padding-bottom: 6px;
}

.graph-rail {
	position: relative;
	display: flex;
	justify-content: center;
	padding-top: 18px;
}

.graph-rail::before {
	position: absolute;
	top: 0;
	bottom: -10px;
	width: 2px;
	border-radius: 999px;
	background: linear-gradient(
		to bottom,
		transparent,
		var(--app-rule-strong) 18px,
		var(--app-rule) calc(100% - 10px),
		transparent
	);
	content: "";
}

.graph-dot {
	z-index: 1;
	width: 12px;
	height: 12px;
	border: 2px solid var(--app-rule-strong);
	border-radius: 999px;
	background: var(--app-surface-elevated);
}

.version-node--current .graph-dot {
	border-color: var(--app-success);
	box-shadow: 0 0 0 3px var(--app-success-soft);
}

.version-node--branching .graph-dot {
	border-color: var(--app-accent);
	box-shadow: 0 0 0 3px var(--app-accent-soft);
}

.version-node-content {
	min-width: 0;
}

.version-card {
	display: grid;
	grid-template-areas: "main metrics actions";
	grid-template-columns: minmax(180px, 0.68fr) minmax(360px, 1.15fr) auto;
	align-items: center;
	gap: 12px;
	min-width: 0;
	padding: 8px 10px;
	border: 1px solid transparent;
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-surface-panel-muted) 54%, transparent);
}

.version-node--current .version-card {
	border-color: color-mix(in srgb, var(--app-success) 28%, var(--app-rule));
	background: color-mix(in srgb, var(--app-success-soft) 46%, var(--app-surface-elevated));
}

.version-node--branching .version-card {
	border-color: color-mix(in srgb, var(--app-accent) 24%, var(--app-rule));
}

.version-card-main {
	grid-area: main;
	min-width: 0;
}

.version-row-header {
	display: grid;
	grid-template-columns: minmax(0, auto) 1fr;
	align-items: baseline;
	gap: 10px;
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
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-muted);
	font-size: 12px;
}

.version-metrics-strip {
	grid-area: metrics;
	display: flex;
	min-width: 0;
	flex-wrap: nowrap;
	align-items: center;
	gap: 4px 8px;
	overflow: hidden;
	font-size: 12px;
	line-height: 1.35;
}

.version-metric {
	display: inline-flex;
	flex: 0 1 132px;
	min-width: 78px;
	max-width: 150px;
	align-items: baseline;
	gap: 4px;
}

.version-metric-label {
	flex: 0 0 auto;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-muted);
	font-weight: 600;
}

.version-metric-label::after {
	content: ":";
}

.version-metric-value {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-ink);
	font-family: var(--app-mono-font);
	font-size: 11px;
}

.version-more-metadata {
	flex: 0 0 auto;
	color: var(--app-subtle);
	font-family: var(--app-mono-font);
	font-size: 11px;
}

.version-empty-metadata {
	grid-area: metrics;
	color: var(--app-subtle);
	font-size: 12px;
}

.versions-empty {
	padding: 36px 0 42px;
}

.version-row-actions {
	grid-area: actions;
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 6px;
}

.branch-editor {
	margin-top: 8px;
	padding: 12px;
	border: 1px solid color-mix(in srgb, var(--app-accent) 24%, var(--app-rule));
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-accent-soft) 50%, transparent);
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

.branch-source {
	margin-bottom: 10px;
	color: var(--app-muted);
	font-size: 12px;
	font-weight: 600;
}

.branch-source span {
	color: var(--app-ink);
	font-family: var(--app-mono-font);
}

.branch-grid {
	display: grid;
	grid-template-columns: minmax(180px, 1fr) minmax(220px, 1.4fr) auto;
	align-items: end;
	gap: 12px;
}

.branch-actions {
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 8px;
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

.branch-editor .clone-mode-row {
	margin-top: 0;
	padding-top: 0;
	border-top: 0;
}

@media (max-width: 1180px) {
	.version-card {
		grid-template-columns: minmax(158px, 0.62fr) minmax(260px, 1fr) auto;
	}

	.version-metric {
		flex-basis: 108px;
		min-width: 66px;
	}

	.version-metric:nth-child(3) {
		display: none;
	}

	.branch-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}

	.branch-actions {
		grid-column: 1 / -1;
	}
}

@media (max-width: 1100px) {
	.version-card {
		grid-template-areas:
			"main actions"
			"metrics metrics";
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 5px 8px;
		padding: 7px 8px;
	}

	.version-id {
		gap: 5px;
	}

	.version-row-header {
		gap: 7px;
	}

	.version-metrics-strip {
		flex-wrap: nowrap;
		gap: 3px 8px;
		overflow: hidden;
	}

	.version-metric {
		flex: 0 1 auto;
		max-width: 92px;
	}

	.version-metric:nth-child(3),
	.version-metric:nth-child(4) {
		display: none;
	}

	.version-metric-label {
		max-width: 58px;
	}

	.version-row-actions {
		align-items: center;
		gap: 4px;
	}

	.version-row-actions :deep(.n-button) {
		padding: 0 7px;
	}
}

@media (max-width: 720px) {
	.version-row-actions,
	.branch-actions {
		flex-direction: column;
		align-items: stretch;
	}

	.versions-actions,
	.version-jump {
		width: 100%;
		justify-content: stretch;
	}

	.version-jump :deep(.n-input-number),
	.version-jump :deep(.n-button) {
		flex: 1 1 0;
	}

	.version-card,
	.branch-grid {
		grid-template-columns: 1fr;
	}

	.version-row-header {
		grid-template-columns: 1fr;
		gap: 4px;
	}

	.version-graph {
		max-height: none;
	}
}
</style>

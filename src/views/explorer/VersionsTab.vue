<script setup lang="ts">
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
	<div class="space-y-4">
		<NCard size="small" title="版本列表" class="shadow-sm">
			<div class="flex flex-wrap items-center justify-between gap-2 text-xs text-slate-500">
				<span>当前版本：{{ currentVersion ?? "—" }}</span>
				<div class="flex items-center gap-2">
					<NButton
						secondary
						:loading="isLoadingVersions"
						:disabled="!hasActiveTable"
						@click="loadVersions"
					>
						刷新版本
					</NButton>
					<NButton
						quaternary
						:disabled="!hasActiveTable"
						@click="loadCurrentVersion"
					>
						刷新当前版本
					</NButton>
				</div>
			</div>
			<NAlert v-if="versionError" type="error" :bordered="false" class="mt-3">
				{{ versionError }}
			</NAlert>
			<div v-if="isLoadingVersions && !versions.length" class="space-y-2 py-4">
				<NSkeleton text :repeat="4" class="w-full" />
			</div>
			<NTimeline
				v-else-if="timelineItems.length"
				class="mt-4 max-h-[320px] overflow-y-auto pl-1 pr-2"
			>
				<NTimelineItem
					v-for="item in timelineItems"
					:key="item.version"
					:type="item.type"
					:title="item.title"
					:time="item.time"
					:line-type="item.isCurrent ? 'dashed' : 'default'"
				>
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
					<div v-else class="text-xs text-slate-400">无 metadata</div>
				</NTimelineItem>
			</NTimeline>
			<NEmpty v-else description="暂无版本记录" class="mt-3" />
		</NCard>

		<NCard size="small" title="打开版本" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-2">
					<label for="checkout-version-input" class="text-sm font-medium text-slate-600">
						版本号
					</label>
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
				</div>
				<div class="xl:col-span-4 flex items-end justify-end gap-2">
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
						回到最新
					</NButton>
				</div>
			</div>
		</NCard>

		<NCard size="small" title="克隆/分支" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">新表名</label>
					<NInput
						v-model:value="cloneTargetName"
						placeholder="clone_table"
						:disabled="!hasActiveTable"
					/>
				</div>
				<div class="xl:col-span-2">
					<label for="clone-source-version-input" class="text-sm font-medium text-slate-600">
						源版本（可选）
					</label>
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
				</div>
				<div class="xl:col-span-1 flex items-end">
					<NCheckbox
						v-model:checked="cloneIsShallow"
						:disabled="!hasActiveTable"
					>
						浅克隆
					</NCheckbox>
				</div>
				<div class="xl:col-span-6 flex items-center justify-end">
					<NButton
						type="primary"
						:loading="isCloningTable"
						:disabled="!hasActiveTable"
						@click="submitCloneTable"
					>
						创建克隆
					</NButton>
				</div>
			</div>
			<div class="mt-2 text-xs text-slate-400">
				浅克隆共享数据文件，适合做分支试验；深克隆暂未实现。
			</div>
		</NCard>
	</div>
</template>

<style scoped>
.version-metadata-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
	gap: 6px;
	margin-top: 6px;
}

.version-metadata-entry {
	display: flex;
	min-width: 0;
	align-items: center;
	gap: 6px;
	border: 1px solid #e2e8f0;
	border-radius: 6px;
	background: #f8fafc;
	padding: 4px 6px;
	font-size: 12px;
	line-height: 1.4;
}

.version-metadata-key {
	flex: 0 0 auto;
	max-width: 45%;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: #64748b;
	font-weight: 600;
}

.version-metadata-value {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: #334155;
}
</style>

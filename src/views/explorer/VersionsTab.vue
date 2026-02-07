<script setup lang="ts">
import { computed, inject, ref, watch } from "vue"
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
import { formatMetadata, TRIGGER_DATA_REFRESH_KEY } from "./explorerShared"

const {
	activeProfileId,
	connectionId,
	activeTableId,
	setError,
	setStatus,
	clearMessages,
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
const isCheckingOutVersion = ref(false)
const isCheckingOutLatest = ref(false)

const timelineItems = computed(() =>
	versions.value.map((v) => {
		const isCurrent = v.version === currentVersion.value
		const meta = formatMetadata(v.metadata)
		return {
			version: v.version,
			isCurrent,
			type: (isCurrent ? "success" : "default") as "success" | "default",
			title: `v${v.version}${isCurrent ? "  (当前)" : ""}`,
			time: formatTimestamp(v.timestamp),
			content: meta !== "—" ? meta : undefined,
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
	if (!profileId || !tableId || isCheckingOutVersion.value) {
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
	try {
		isCheckingOutVersion.value = true
		clearMessages()
		const response = unwrapEnvelope(await checkoutTableVersionV1({ tableId, version }))
		currentVersion.value = response.version
		setStatus(`已切换到版本 ${response.version}`)
		await refreshSchema(profileId)
		triggerDataRefresh()
	} catch (error) {
		setError(error instanceof Error ? error.message : "切换版本失败")
	} finally {
		isCheckingOutVersion.value = false
	}
}

async function submitCheckoutLatest() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	if (!profileId || !tableId || isCheckingOutLatest.value) {
		return
	}
	try {
		isCheckingOutLatest.value = true
		clearMessages()
		const response = unwrapEnvelope(await checkoutTableLatestV1({ tableId }))
		currentVersion.value = response.version
		setStatus(`已回到最新版本 ${response.version}`)
		await refreshSchema(profileId)
		triggerDataRefresh()
	} catch (error) {
		setError(error instanceof Error ? error.message : "恢复最新版本失败")
	} finally {
		isCheckingOutLatest.value = false
	}
}

// ── Clone ──────────────────────────────────────────────

const cloneTargetName = ref("")
const cloneSourceVersion = ref<number | null>(null)
const cloneIsShallow = ref(true)
const isCloningTable = ref(false)

async function submitCloneTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableId = activeTableId.value
	if (!profileId || !currentConnectionId || !tableId || isCloningTable.value) {
		return
	}
	const targetName = cloneTargetName.value.trim()
	if (!targetName) {
		setError("请输入克隆表名")
		return
	}
	try {
		isCloningTable.value = true
		clearMessages()
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
	} catch (error) {
		setError(error instanceof Error ? error.message : "克隆表失败")
	} finally {
		isCloningTable.value = false
	}
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
					:content="item.content"
					:line-type="item.isCurrent ? 'dashed' : 'default'"
				/>
			</NTimeline>
			<NEmpty v-else description="暂无版本记录" class="mt-3" />
		</NCard>

		<NCard size="small" title="打开版本" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-2">
					<label class="text-sm font-medium text-slate-600">版本号</label>
					<NInputNumber
						v-model:value="checkoutVersion"
						:min="0"
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
					<label class="text-sm font-medium text-slate-600">源版本（可选）</label>
					<NInputNumber
						v-model:value="cloneSourceVersion"
						:min="0"
						placeholder="留空使用最新"
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

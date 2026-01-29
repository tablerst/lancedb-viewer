<script setup lang="ts">
import { type DataTableColumns, NButton, NPopconfirm, NTag } from "naive-ui"
import { computed, h, onMounted, ref } from "vue"
import { useRouter } from "vue-router"

import { useWorkspace } from "../composables/workspaceContext"
import { collectCredentialReferences } from "../lib/credentialReferences"
import type { CredentialSummary } from "../lib/credentialVault"
import {
	cleanupUnusedCredentials,
	clearCredentials,
	deleteCredential,
	listCredentials,
} from "../lib/credentialVault"
import { formatTimestamp } from "../lib/formatters"

type CredentialRow = CredentialSummary & { used: boolean }

const router = useRouter()

const { profiles, activeProfileId, activeProfile, setStatus, setError, clearMessages } =
	useWorkspace()

const credentials = ref<CredentialSummary[]>([])
const isLoading = ref(false)
const isCleaning = ref(false)
const isClearing = ref(false)
const deletingReference = ref<string | null>(null)
const errorMessage = ref("")
const lastLoadedAt = ref<string | null>(null)

const usedReferences = computed(() => collectCredentialReferences(profiles.value))

const tableData = computed<CredentialRow[]>(() =>
	credentials.value.map((item) => ({
		...item,
		used: usedReferences.value.has(item.reference),
	}))
)

const referencedCount = computed(() => tableData.value.filter((item) => item.used).length)
const unusedCount = computed(() => tableData.value.filter((item) => !item.used).length)

function openActiveConnectionCredentials() {
	const id = activeProfileId.value
	if (!id) {
		setError("请先选择连接")
		return
	}
	void router.push(`/connections/${id}/credentials`)
}

async function loadCredentials() {
	if (isLoading.value) {
		return
	}
	isLoading.value = true
	errorMessage.value = ""
	try {
		credentials.value = await listCredentials()
		lastLoadedAt.value = new Date().toISOString()
	} catch (error) {
		const message = error instanceof Error ? error.message : "读取凭证失败"
		errorMessage.value = message
		setError(message)
	} finally {
		isLoading.value = false
	}
}

async function handleDelete(reference: string) {
	if (deletingReference.value) {
		return
	}
	if (usedReferences.value.has(reference)) {
		setError("凭证仍被引用，无法删除")
		return
	}
	deletingReference.value = reference
	try {
		clearMessages()
		await deleteCredential(reference)
		await loadCredentials()
		setStatus("已删除凭证")
	} catch (error) {
		const message = error instanceof Error ? error.message : "删除凭证失败"
		setError(message)
	} finally {
		deletingReference.value = null
	}
}

async function handleCleanup() {
	if (isCleaning.value) {
		return
	}
	isCleaning.value = true
	try {
		clearMessages()
		const removed = await cleanupUnusedCredentials(usedReferences.value)
		await loadCredentials()
		if (removed.length > 0) {
			setStatus(`已清理 ${removed.length} 个未引用凭证`)
		} else {
			setStatus("暂无可清理的凭证")
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : "清理凭证失败"
		setError(message)
	} finally {
		isCleaning.value = false
	}
}

async function handleClearAll() {
	if (isClearing.value) {
		return
	}
	isClearing.value = true
	try {
		clearMessages()
		await clearCredentials()
		await loadCredentials()
		setStatus("已清空 Stronghold 凭证")
	} catch (error) {
		const message = error instanceof Error ? error.message : "清空凭证失败"
		setError(message)
	} finally {
		isClearing.value = false
	}
}

const columns: DataTableColumns<CredentialRow> = [
	{
		title: "标签",
		key: "label",
		ellipsis: { tooltip: true },
		render: (row) => row.label ?? "—",
	},
	{
		title: "Provider",
		key: "provider",
		ellipsis: { tooltip: true },
	},
	{
		title: "引用",
		key: "reference",
		ellipsis: { tooltip: true },
		render: (row) => h("span", { class: "font-mono text-xs text-slate-600" }, row.reference),
	},
	{
		title: "更新时间",
		key: "updatedAt",
		ellipsis: { tooltip: true },
		render: (row) => formatTimestamp(row.updatedAt),
	},
	{
		title: "状态",
		key: "used",
		render: (row) =>
			h(
				NTag,
				{ size: "small", type: row.used ? "success" : "warning" },
				{ default: () => (row.used ? "引用中" : "未引用") }
			),
	},
	{
		title: "操作",
		key: "actions",
		render: (row) => {
			if (row.used) {
				return h("span", { class: "text-xs text-slate-400" }, "引用中")
			}
			return h(
				NPopconfirm,
				{
					positiveText: "删除",
					negativeText: "取消",
					onPositiveClick: () => handleDelete(row.reference),
				},
				{
					default: () => "确定删除该凭证吗？",
					trigger: () =>
						h(
							NButton,
							{
								size: "tiny",
								type: "error",
								secondary: true,
								loading: deletingReference.value === row.reference,
							},
							{ default: () => "删除" }
						),
				}
			)
		},
	},
]

onMounted(() => {
	void loadCredentials()
})
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="凭证库（高级）" class="shadow-sm">
			<div class="text-xs text-slate-500">
				这是 Stronghold 的全局凭证库，用于查看/回收未引用凭证。日常配置建议在“连接 → 凭证”中完成。
			</div>
			<div class="flex flex-wrap items-center justify-between gap-2">
				<div class="text-xs text-slate-500">
					总数：{{ credentials.length }} · 引用中：{{ referencedCount }} · 未引用：{{ unusedCount }}
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<NButton
						size="small"
						:disabled="!activeProfileId"
						@click="openActiveConnectionCredentials"
					>
						去当前连接配置（{{ activeProfile?.name ?? "未选择" }}）
					</NButton>
					<NButton size="small" :loading="isLoading" @click="loadCredentials">
						刷新
					</NButton>
					<NButton
						size="small"
						type="warning"
						:loading="isCleaning"
						:disabled="unusedCount === 0"
						@click="handleCleanup"
					>
						清理未引用
					</NButton>
					<NPopconfirm
						positive-text="清空"
						negative-text="取消"
						@positive-click="handleClearAll"
					>
						<template #trigger>
							<NButton
								size="small"
								type="error"
								secondary
								:loading="isClearing"
							>
								清空 Stronghold
							</NButton>
						</template>
						清空后所有引用将失效，确定继续吗？
					</NPopconfirm>
				</div>
			</div>
			<div class="mt-2 text-xs text-slate-400">
				引用回收策略：连接档案更新/删除后自动清理未引用凭证；可在此手动清理。
			</div>
			<div v-if="lastLoadedAt" class="mt-1 text-[11px] text-slate-400">
				上次刷新：{{ formatTimestamp(lastLoadedAt) }}
			</div>
		</NCard>

		<NAlert v-if="errorMessage" type="error" :bordered="false">
			{{ errorMessage }}
		</NAlert>

		<NEmpty v-if="!tableData.length && !isLoading" description="暂无保存的凭证" />

		<NDataTable
			v-else
			class="data-table"
			size="small"
			:columns="columns"
			:data="tableData"
			:loading="isLoading"
			:bordered="false"
			:row-key="(row) => row.reference"
		/>
	</div>
</template>

<style scoped>
.data-table :deep(.n-data-table-th),
.data-table :deep(.n-data-table-td) {
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}
</style>

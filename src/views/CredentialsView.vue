<script setup lang="ts">
import {
	Copy,
	ExternalLink,
	KeyRound,
	MoreHorizontal,
	RefreshCcw,
	Settings2,
	ShieldCheck,
	ShieldOff,
	Trash2,
} from "lucide-vue-next"
import { useDialog } from "naive-ui"
import type { DropdownMixedOption } from "naive-ui/lib/dropdown/src/interface"
import { computed, h, onMounted, ref, shallowRef } from "vue"
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

type CredentialSource = "stronghold" | "mock"
type CredentialRow = CredentialSummary & { used: boolean; source: CredentialSource }
type CredentialCard = CredentialRow & {
	referencedBy: Array<{ id: string; name: string }>
}
type CredentialMenuKey = "copy-reference" | "open-profile" | "delete"

const router = useRouter()
const dialog = useDialog()

const { profiles, activeProfileId, setStatus, setError, clearMessages } = useWorkspace()

const credentials = ref<CredentialSummary[]>([])
const isLoading = shallowRef(false)
const isCleaning = shallowRef(false)
const isClearing = shallowRef(false)
const deletingReference = shallowRef<string | null>(null)
const errorMessage = shallowRef("")
const lastLoadedAt = shallowRef<string | null>(null)
const isPreviewingMock = shallowRef(false)

const usedReferences = computed(() => collectCredentialReferences(profiles.value))

const mockCredentialRows: CredentialRow[] = [
	{
		reference: "cred_mock_s3_prod_readonly_20260618",
		provider: "s3",
		label: "生产 S3 只读密钥",
		updatedAt: "2026-06-18T08:30:00.000Z",
		used: true,
		source: "mock",
	},
	{
		reference: "cred_mock_gcs_lab_orphaned_20260618",
		provider: "gcs",
		label: "实验桶临时凭证",
		updatedAt: "2026-06-18T07:12:00.000Z",
		used: false,
		source: "mock",
	},
]

const realTableData = computed<CredentialRow[]>(() =>
	credentials.value.map((item) => ({
		...item,
		used: usedReferences.value.has(item.reference),
		source: "stronghold" as const,
	}))
)

const tableData = computed<CredentialRow[]>(() =>
	isPreviewingMock.value ? mockCredentialRows : realTableData.value
)

const credentialCards = computed<CredentialCard[]>(() =>
	tableData.value.map((item) => ({
		...item,
		referencedBy: profiles.value
			.filter((profile) => {
				const auth = profile.auth
				return auth?.type === "secret_ref" && auth.reference.trim() === item.reference
			})
			.map((profile) => ({ id: profile.id, name: profile.name })),
	}))
)

const referencedCount = computed(() => tableData.value.filter((item) => item.used).length)
const unusedCount = computed(() => tableData.value.filter((item) => !item.used).length)
const sourceLabel = computed(() => (isPreviewingMock.value ? "Mock 示例" : "Stronghold"))

function toggleMockPreview() {
	isPreviewingMock.value = !isPreviewingMock.value
	errorMessage.value = ""
}

function openActiveConnectionCredentials() {
	const id = activeProfileId.value
	if (!id) {
		setError("请先选择连接")
		return
	}
	void router.push(`/connections/${id}/credentials`)
}

function openConnectionCredentials(id: string) {
	void router.push(`/connections/${id}/credentials`)
}

async function copyCredentialReference(reference: string) {
	if (!navigator.clipboard?.writeText) {
		setError("当前环境不支持复制引用")
		return
	}
	try {
		await navigator.clipboard.writeText(reference)
		setStatus("已复制凭证引用")
	} catch {
		setError("复制凭证引用失败")
	}
}

function getCredentialMenuOptions(row: CredentialCard): DropdownMixedOption[] {
	const referencedProfile = row.referencedBy[0] ?? null
	const canOpenProfile = Boolean(referencedProfile || activeProfileId.value)
	const canDelete = row.source === "stronghold" && !row.used

	return [
		{
			key: "copy-reference",
			label: "复制引用",
			icon: () => h(Copy, { class: "h-4 w-4" }),
		},
		{
			key: "open-profile",
			label: referencedProfile ? "打开引用连接凭证" : "打开当前连接凭证",
			disabled: !canOpenProfile,
			icon: () => h(ExternalLink, { class: "h-4 w-4" }),
		},
		{ type: "divider", key: "danger-divider" },
		{
			key: "delete",
			label: () =>
				h(
					"span",
					{
						style: canDelete
							? { color: "var(--app-danger)", fontWeight: 500 }
							: { color: "var(--app-subtle)" },
					},
					"删除凭证..."
				),
			disabled: !canDelete,
			icon: () =>
				h(Trash2, {
					class: "h-4 w-4",
					style: canDelete ? { color: "var(--app-danger)" } : undefined,
				}),
		},
	]
}

function confirmDeleteCredential(row: CredentialCard) {
	if (row.source === "mock") {
		setStatus("当前为 Mock 预览，未删除 Stronghold 凭证")
		return
	}
	if (row.used) {
		setError("凭证仍被引用，无法删除")
		return
	}
	dialog.warning({
		title: "删除未引用凭证",
		content: `确定删除 ${row.label ?? row.reference} 吗？删除后无法恢复。`,
		positiveText: "删除",
		negativeText: "取消",
		positiveButtonProps: { type: "error" },
		onPositiveClick: async () => {
			await handleDelete(row.reference)
		},
	})
}

function handleCredentialMenuSelect(rawKey: string | number, row: CredentialCard) {
	const key = String(rawKey) as CredentialMenuKey
	switch (key) {
		case "copy-reference":
			void copyCredentialReference(row.reference)
			break
		case "open-profile": {
			const profileId = row.referencedBy[0]?.id ?? activeProfileId.value
			if (!profileId) {
				setError("请先选择连接")
				return
			}
			openConnectionCredentials(profileId)
			break
		}
		case "delete":
			confirmDeleteCredential(row)
			break
		default:
			break
	}
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
	if (isPreviewingMock.value) {
		setStatus("当前为 Mock 预览，未删除 Stronghold 凭证")
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
	if (isPreviewingMock.value) {
		setStatus("当前为 Mock 预览，未清理 Stronghold 凭证")
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
	if (isPreviewingMock.value) {
		setStatus("当前为 Mock 预览，未清空 Stronghold 凭证")
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

onMounted(() => {
	void loadCredentials()
})
</script>

<template>
	<div class="credentials-view">
		<section class="credentials-overview" aria-labelledby="credentials-title">
			<div class="credentials-intro">
				<div class="credentials-title-group">
					<div class="credentials-kicker">Credential Vault</div>
					<h1 id="credentials-title" class="credentials-title">凭证库</h1>
					<p class="credentials-description">
						查看 Stronghold 保存的凭证引用，判断哪些仍被连接档案使用。日常配置仍建议从“连接
						→ 凭证”进入。
					</p>
				</div>

				<div class="credentials-source-panel">
					<span class="credentials-panel-label">当前来源</span>
					<strong>{{ sourceLabel }}</strong>
					<span>
						{{ isPreviewingMock ? `真实库 ${credentials.length} 条` : "Stronghold 实时索引" }}
					</span>
				</div>
			</div>

			<div class="credentials-metrics">
				<div class="credentials-metric">
					<span class="credentials-metric-label">总数</span>
					<strong>{{ tableData.length }}</strong>
					<span class="credentials-metric-note">当前显示项</span>
				</div>
				<div class="credentials-metric">
					<span class="credentials-metric-label">引用中</span>
					<strong>{{ referencedCount }}</strong>
					<span class="credentials-metric-note">仍受连接档案保护</span>
				</div>
				<div class="credentials-metric">
					<span class="credentials-metric-label">未引用</span>
					<strong>{{ unusedCount }}</strong>
					<span class="credentials-metric-note">可手动清理</span>
				</div>
			</div>

			<div class="credentials-command-bar" aria-label="凭证库批量操作">
				<div class="credentials-command-text">
					<span>批量操作</span>
					<strong v-if="lastLoadedAt">上次刷新 {{ formatTimestamp(lastLoadedAt) }}</strong>
					<strong v-else>等待首次刷新</strong>
				</div>
				<div class="credentials-actions">
					<NButton
						size="small"
						type="primary"
						secondary
						:disabled="!activeProfileId"
						@click="openActiveConnectionCredentials"
					>
						<template #icon>
							<Settings2 class="h-4 w-4" />
						</template>
						当前连接凭证
					</NButton>
					<NButton size="small" :loading="isLoading" @click="loadCredentials">
						<template #icon>
							<RefreshCcw class="h-4 w-4" />
						</template>
						刷新
					</NButton>
					<NButton size="small" secondary @click="toggleMockPreview">
						{{ isPreviewingMock ? "关闭 Mock" : "预览 Mock" }}
					</NButton>
					<NButton
						size="small"
						type="warning"
						:loading="isCleaning"
						:disabled="isPreviewingMock || unusedCount === 0"
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
								:disabled="isPreviewingMock"
							>
								清空 Stronghold
							</NButton>
						</template>
						清空后所有引用将失效，确定继续吗？
					</NPopconfirm>
				</div>
			</div>
		</section>

		<NAlert v-if="isPreviewingMock" type="info" :bordered="false" class="mock-preview-alert">
			当前显示两条 Mock 凭证，仅用于观察列表态；删除、清理和清空不会作用到 Stronghold。
		</NAlert>

		<NAlert v-if="errorMessage" type="error" :bordered="false">
			{{ errorMessage }}
		</NAlert>

		<div v-if="!credentialCards.length && !isLoading" class="credentials-empty">
			<NEmpty description="暂无保存的凭证">
				<template #extra>
					<div class="credentials-empty-actions">
						<NButton size="small" @click="loadCredentials">刷新</NButton>
						<NButton size="small" secondary @click="toggleMockPreview">
							载入 2 条 Mock 数据
						</NButton>
						<NButton
							size="small"
							type="primary"
							:disabled="!activeProfileId"
							@click="openActiveConnectionCredentials"
						>
							配置当前连接
						</NButton>
					</div>
				</template>
			</NEmpty>
		</div>

		<div v-else class="credentials-grid-flow" aria-label="凭证列表">
			<article
				v-for="item in credentialCards"
				:key="item.reference"
				class="credential-card"
				:class="{
					'credential-card--used': item.used,
					'credential-card--unused': !item.used,
					'credential-card--mock': item.source === 'mock',
				}"
			>
				<div class="credential-card-header">
					<div class="credential-card-identity">
						<div class="credential-card-icon" aria-hidden="true">
							<ShieldCheck v-if="item.used" class="h-4 w-4" />
							<ShieldOff v-else class="h-4 w-4" />
						</div>
						<div class="credential-card-title-block">
							<h2 class="credential-card-title">{{ item.label ?? "未命名凭证" }}</h2>
							<div class="credential-card-subtitle">
								<span>{{ item.provider || "unknown provider" }}</span>
								<span aria-hidden="true">/</span>
								<span>{{ item.source === "mock" ? "Mock" : "Stronghold" }}</span>
							</div>
						</div>
					</div>

					<NDropdown
						trigger="click"
						placement="bottom-end"
						:show-arrow="false"
						:options="getCredentialMenuOptions(item)"
						@select="(key) => handleCredentialMenuSelect(key, item)"
					>
						<NButton
							size="tiny"
							quaternary
							circle
							:aria-label="`打开 ${item.label ?? item.reference} 操作菜单`"
							:title="`打开 ${item.label ?? item.reference} 操作菜单`"
						>
							<MoreHorizontal class="h-4 w-4" />
						</NButton>
					</NDropdown>
				</div>

				<div class="credential-card-body">
					<div class="credential-reference-line">
						<KeyRound class="h-3.5 w-3.5" aria-hidden="true" />
						<code>{{ item.reference }}</code>
					</div>

					<div class="credential-card-facts">
						<div>
							<span>状态</span>
							<NTag size="small" :type="item.used ? 'success' : 'warning'">
								{{ item.used ? "引用中" : "未引用" }}
							</NTag>
						</div>
						<div>
							<span>更新时间</span>
							<strong>{{ formatTimestamp(item.updatedAt) }}</strong>
						</div>
					</div>
				</div>

				<div class="credential-card-footer">
					<div class="credential-profile-strip">
						<span v-if="item.referencedBy.length" class="credential-profile-label">引用连接</span>
						<span v-else class="credential-profile-label">未被连接引用</span>
						<div v-if="item.referencedBy.length" class="credential-profile-tags">
							<NButton
								v-for="profile in item.referencedBy.slice(0, 2)"
								:key="profile.id"
								size="tiny"
								secondary
								@click="openConnectionCredentials(profile.id)"
							>
								{{ profile.name }}
							</NButton>
							<span v-if="item.referencedBy.length > 2" class="credential-profile-more">
								+{{ item.referencedBy.length - 2 }}
							</span>
						</div>
					</div>

					<NButton
						v-if="item.source === 'stronghold' && !item.used"
						size="tiny"
						type="error"
						secondary
						:loading="deletingReference === item.reference"
						@click="confirmDeleteCredential(item)"
					>
						删除
					</NButton>
				</div>
			</article>
		</div>
	</div>
</template>

<style scoped>
.credentials-view {
	display: flex;
	flex-direction: column;
	gap: 14px;
}

.credentials-overview {
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
	padding: 16px;
	box-shadow: var(--app-shadow-whisper);
}

.credentials-intro {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(180px, 240px);
	gap: 18px;
	align-items: flex-start;
}

.credentials-title-group {
	min-width: 0;
	max-width: 680px;
}

.credentials-kicker {
	color: var(--app-accent-strong);
	font-size: 11px;
	font-weight: 700;
	letter-spacing: 0;
}

.credentials-title {
	margin: 2px 0 0;
	color: var(--app-ink-strong);
	font-size: 20px;
	font-weight: 720;
	line-height: 1.25;
}

.credentials-description {
	margin: 6px 0 0;
	color: var(--app-muted);
	font-size: 12px;
	line-height: 1.7;
}

.credentials-source-panel {
	display: flex;
	min-width: 0;
	flex-direction: column;
	gap: 2px;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-accent-soft) 58%, transparent);
	padding: 10px 12px;
}

.credentials-panel-label {
	color: var(--app-muted);
	font-size: 11px;
	line-height: 1.35;
}

.credentials-source-panel strong {
	color: var(--app-ink-strong);
	font-size: 16px;
	font-weight: 720;
	line-height: 1.35;
	overflow-wrap: anywhere;
}

.credentials-source-panel span:last-child {
	color: var(--app-muted);
	font-size: 11px;
	line-height: 1.35;
}

.credentials-actions {
	display: flex;
	flex-wrap: wrap;
	gap: 8px;
	justify-content: flex-end;
}

.credentials-metrics {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
	gap: 8px;
	margin-top: 16px;
}

.credentials-metric {
	min-width: 0;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-surface-panel-muted) 58%, transparent);
	padding: 10px 12px;
}

.credentials-metric-label,
.credentials-metric-note {
	display: block;
	color: var(--app-muted);
	font-size: 11px;
	line-height: 1.45;
}

.credentials-metric strong {
	display: block;
	margin-top: 2px;
	color: var(--app-ink-strong);
	font-size: 18px;
	font-weight: 720;
	line-height: 1.2;
	overflow-wrap: anywhere;
}

.credentials-command-bar {
	display: grid;
	grid-template-columns: minmax(0, 1fr) auto;
	gap: 12px;
	align-items: center;
	margin-top: 12px;
	border-top: 1px solid var(--app-rule);
	padding-top: 12px;
}

.credentials-command-text {
	display: flex;
	min-width: 0;
	flex-direction: column;
	gap: 2px;
	color: var(--app-ink);
	font-size: 11px;
	line-height: 1.5;
}

.credentials-command-text span {
	color: var(--app-muted);
}

.credentials-command-text strong {
	color: var(--app-subtle);
	font-weight: 500;
	overflow-wrap: anywhere;
}

.mock-preview-alert {
	border: 1px solid color-mix(in srgb, var(--app-accent) 28%, var(--app-rule));
}

.credentials-grid-flow {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
	gap: 12px;
}

.credential-card {
	position: relative;
	display: flex;
	min-width: 0;
	min-height: 236px;
	flex-direction: column;
	justify-content: space-between;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: var(--app-surface-elevated);
	padding: 14px;
	box-shadow: var(--app-shadow-whisper);
	transition:
		border-color 120ms ease,
		background-color 120ms ease,
		transform 120ms ease;
}

.credential-card:hover {
	border-color: var(--app-rule-strong);
	background: color-mix(in srgb, var(--app-surface-elevated) 90%, var(--app-surface-panel));
	transform: translateY(-1px);
}

.credential-card--unused {
	border-color: color-mix(in srgb, var(--app-warning) 24%, var(--app-rule));
}

.credential-card--mock {
	border-style: dashed;
}

.credential-card-header {
	display: flex;
	min-width: 0;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
}

.credential-card-identity {
	display: flex;
	min-width: 0;
	align-items: flex-start;
	gap: 10px;
}

.credential-card-icon {
	display: inline-flex;
	width: 30px;
	height: 30px;
	flex: 0 0 auto;
	align-items: center;
	justify-content: center;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
	background: var(--app-surface-panel-muted);
	color: var(--app-accent-strong);
}

.credential-card--unused .credential-card-icon {
	background: var(--app-warning-soft);
	color: var(--app-warning);
}

.credential-card-title-block {
	min-width: 0;
}

.credential-card-title {
	margin: 0;
	color: var(--app-ink-strong);
	font-size: 14px;
	font-weight: 700;
	line-height: 1.35;
	overflow-wrap: anywhere;
}

.credential-card-subtitle {
	display: flex;
	min-width: 0;
	flex-wrap: wrap;
	gap: 5px;
	margin-top: 3px;
	color: var(--app-muted);
	font-size: 11px;
	line-height: 1.4;
}

.credential-card-subtitle span {
	min-width: 0;
	overflow-wrap: anywhere;
}

.credential-card-body {
	display: flex;
	flex-direction: column;
	gap: 12px;
	margin-top: 14px;
}

.credential-reference-line {
	display: grid;
	grid-template-columns: auto minmax(0, 1fr);
	gap: 7px;
	align-items: center;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
	background: var(--app-surface-panel-muted);
	padding: 8px;
	color: var(--app-muted);
}

.credential-reference-line code {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	color: var(--app-ink);
	font-family: var(--app-mono-font);
	font-size: 11px;
}

.credential-card-facts {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 8px;
}

.credential-card-facts > div {
	display: flex;
	min-width: 0;
	align-items: flex-start;
	flex-direction: column;
	gap: 5px;
}

.credential-card-facts span {
	color: var(--app-muted);
	font-size: 11px;
	line-height: 1.35;
}

.credential-card-facts strong {
	color: var(--app-ink);
	font-size: 12px;
	font-weight: 560;
	line-height: 1.35;
	overflow-wrap: anywhere;
}

.credential-card-footer {
	display: flex;
	min-width: 0;
	align-items: flex-end;
	justify-content: space-between;
	gap: 12px;
	margin-top: 14px;
	border-top: 1px solid var(--app-rule);
	padding-top: 12px;
}

.credential-profile-strip {
	display: flex;
	min-width: 0;
	flex: 1 1 auto;
	flex-direction: column;
	gap: 6px;
}

.credential-profile-label {
	color: var(--app-subtle);
	font-size: 11px;
	line-height: 1.35;
}

.credential-profile-tags {
	display: flex;
	min-width: 0;
	flex-wrap: wrap;
	gap: 6px;
}

.credential-profile-tags :deep(.n-button__content) {
	min-width: 0;
	max-width: 140px;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.credential-profile-more {
	display: inline-flex;
	align-items: center;
	min-height: 22px;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-sm);
	padding: 0 7px;
	color: var(--app-muted);
	font-size: 11px;
}

.credentials-empty {
	display: flex;
	min-height: 300px;
	align-items: center;
	justify-content: center;
	border: 1px dashed var(--app-rule);
	border-radius: var(--app-radius-lg);
	background: color-mix(in srgb, var(--app-surface-elevated) 70%, var(--app-surface-panel));
}

.credentials-empty-actions {
	display: flex;
	flex-wrap: wrap;
	justify-content: center;
	gap: 8px;
}

@media (max-width: 760px) {
	.credentials-intro,
	.credentials-command-bar {
		grid-template-columns: 1fr;
	}

	.credentials-source-panel,
	.credentials-actions {
		width: 100%;
	}

	.credentials-actions {
		justify-content: flex-start;
	}

	.credentials-metrics {
		grid-template-columns: 1fr;
	}

	.credentials-grid-flow {
		grid-template-columns: 1fr;
	}
}

@media (max-width: 430px) {
	.credential-card {
		min-height: 0;
		padding: 12px;
	}

	.credential-card-facts,
	.credential-card-footer {
		grid-template-columns: 1fr;
	}

	.credential-card-footer {
		align-items: stretch;
		flex-direction: column;
	}

	.credential-card-footer :deep(.n-button) {
		align-self: flex-start;
	}
}
</style>

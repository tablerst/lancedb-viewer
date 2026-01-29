<script setup lang="ts">
import type { SelectOption } from "naive-ui"
import { computed, onMounted, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useWorkspace } from "../composables/workspaceContext"
import type { AuthDescriptor } from "../ipc/v1"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "../lib/connectionKind"
import type { CredentialRecord, CredentialSummary } from "../lib/credentialVault"
import { getCredential, listCredentials, saveCredential } from "../lib/credentialVault"

const route = useRoute()
const router = useRouter()

const {
	profiles,
	activeProfileId,
	activeProfile,
	setStatus,
	setError,
	clearMessages,
	selectProfile,
	updateProfile,
} = useWorkspace()

const routeProfileId = computed(() => {
	const raw = route.params.id
	return typeof raw === "string" ? raw : null
})

const resolvedProfile = computed(() => {
	const id = routeProfileId.value
	if (!id) {
		return activeProfile.value
	}
	return profiles.value.find((profile) => profile.id === id) ?? null
})

const kind = computed(() => {
	const profile = resolvedProfile.value
	return profile ? getConnectionKind(profile.uri) : "unknown"
})

const kindLabel = computed(() => getConnectionKindLabel(kind.value))
const kindTagType = computed(() => getConnectionKindTagType(kind.value))

const savedAuth = computed<AuthDescriptor>(() => resolvedProfile.value?.auth ?? { type: "none" })

const form = ref({
	enabled: false,
	provider: "",
	paramsJson: "{}",
	saveToStronghold: true,
	reference: "",
})

const errorMessage = ref("")
const isSaving = ref(false)
const vaultCredentials = ref<CredentialSummary[]>([])
const vaultLoadError = ref("")
const selectedCredential = ref<CredentialRecord | null>(null)

function parseAuthParams(raw: string): Record<string, string> {
	if (!raw.trim()) {
		return {}
	}
	const parsed = JSON.parse(raw) as Record<string, unknown>
	if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") {
		throw new Error("auth params 必须是 JSON 对象")
	}
	return Object.fromEntries(Object.entries(parsed).map(([key, value]) => [key, String(value)]))
}

function applyFromSavedAuth(auth: AuthDescriptor) {
	if (auth.type === "none") {
		form.value = {
			enabled: false,
			provider: "",
			paramsJson: "{}",
			saveToStronghold: true,
			reference: "",
		}
		return
	}
	if (auth.type === "inline") {
		form.value = {
			enabled: true,
			provider: auth.provider ?? "",
			paramsJson: JSON.stringify(auth.params ?? {}, null, 2),
			saveToStronghold: false,
			reference: "",
		}
		return
	}
	form.value = {
		enabled: true,
		provider: auth.provider ?? "",
		paramsJson: "{}",
		saveToStronghold: true,
		reference: auth.reference ?? "",
	}
}

async function ensureSelected() {
	const id = routeProfileId.value
	if (!id) {
		return
	}
	if (activeProfileId.value === id) {
		return
	}
	const exists = profiles.value.some((profile) => profile.id === id)
	if (!exists) {
		setError("未找到该连接档案")
		return
	}
	await selectProfile(id)
}

const savedAuthTitle = computed(() => {
	switch (savedAuth.value.type) {
		case "none":
			return "未配置认证"
		case "inline":
			return `Inline · ${savedAuth.value.provider || "未指定 provider"}`
		case "secret_ref":
			return `Stronghold 引用 · ${savedAuth.value.provider || "未指定 provider"}`
	}
})

const savedReference = computed(() =>
	savedAuth.value.type === "secret_ref" ? savedAuth.value.reference.trim() : ""
)

const referencedByProfiles = computed(() => {
	const reference = savedReference.value
	if (!reference) {
		return []
	}
	const matched: Array<{ id: string; name: string }> = []
	for (const profile of profiles.value) {
		const auth = profile.auth
		if (!auth || auth.type !== "secret_ref") {
			continue
		}
		if (auth.reference.trim() !== reference) {
			continue
		}
		matched.push({ id: profile.id, name: profile.name })
	}
	return matched
})

const vaultOptions = computed<SelectOption[]>(() =>
	vaultCredentials.value.map((item) => ({
		label: `${item.label ?? item.reference} · ${item.provider}`,
		value: item.reference,
	}))
)

async function loadVaultCredentials() {
	vaultLoadError.value = ""
	try {
		vaultCredentials.value = await listCredentials()
	} catch (error) {
		vaultLoadError.value = error instanceof Error ? error.message : "读取凭证库失败"
	}
}

async function loadSelectedCredential(reference: string) {
	selectedCredential.value = null
	if (!reference.trim()) {
		return
	}
	try {
		selectedCredential.value = await getCredential(reference)
	} catch {
		selectedCredential.value = null
	}
}

function resetToSaved() {
	errorMessage.value = ""
	applyFromSavedAuth(savedAuth.value)
	void loadSelectedCredential(form.value.reference)
}

async function save() {
	const profile = resolvedProfile.value
	if (!profile) {
		setError("请先选择连接")
		return
	}
	if (isSaving.value) {
		return
	}
	await ensureSelected()

	errorMessage.value = ""
	clearMessages()
	isSaving.value = true
	try {
		let nextAuth: AuthDescriptor = { type: "none" }
		if (form.value.enabled) {
			const provider = form.value.provider.trim()
			if (!provider) {
				errorMessage.value = "请填写认证 Provider"
				return
			}
			const rawParams = form.value.paramsJson.trim()
			const paramsProvided = rawParams !== "" && rawParams !== "{}"
			const params = parseAuthParams(form.value.paramsJson)

			if (form.value.saveToStronghold) {
				const reference = form.value.reference.trim()
				if (!paramsProvided && reference) {
					nextAuth = { type: "secret_ref", provider, reference }
				} else {
					const nextReference = await saveCredential({
						provider,
						params,
						label: profile.name,
						reference: reference || undefined,
					})
					form.value.reference = nextReference
					nextAuth = { type: "secret_ref", provider, reference: nextReference }
				}
			} else {
				if (!paramsProvided) {
					errorMessage.value = "请填写凭证参数，或开启 Stronghold 保存"
					return
				}
				nextAuth = { type: "inline", provider, params }
			}
		}

		await updateProfile({
			id: profile.id,
			name: profile.name,
			uri: profile.uri,
			storageOptionsJson: JSON.stringify(profile.storageOptions ?? {}, null, 2),
			auth: nextAuth,
		})

		setStatus("已更新连接凭证")
		await loadVaultCredentials()
		await loadSelectedCredential(form.value.reference)
	} catch (error) {
		const message = error instanceof Error ? error.message : "更新连接凭证失败"
		errorMessage.value = message
		setError(message)
	} finally {
		isSaving.value = false
	}
}

function openVault() {
	void router.push("/vault/credentials")
}

watch(
	() => resolvedProfile.value?.id,
	() => {
		applyFromSavedAuth(savedAuth.value)
		void loadSelectedCredential(form.value.reference)
	},
	{ immediate: true }
)

watch(
	() => form.value.reference,
	(reference) => {
		const meta = vaultCredentials.value.find((item) => item.reference === reference)
		const provider = meta?.provider
		if (provider && form.value.provider.trim() !== provider) {
			form.value.provider = provider
		}
		void loadSelectedCredential(reference)
	}
)

onMounted(() => {
	void loadVaultCredentials()
})
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="连接凭证" class="shadow-sm">
			<div class="space-y-3">
				<div class="flex flex-wrap items-center justify-between gap-2">
					<div class="min-w-0">
						<div class="flex items-center gap-2">
							<div class="truncate text-sm font-semibold text-slate-800">
								{{ resolvedProfile?.name ?? "未选择连接" }}
							</div>
							<NTag v-if="resolvedProfile" size="small" :type="kindTagType">
								{{ kindLabel }}
							</NTag>
						</div>
						<div v-if="resolvedProfile" class="mt-1 truncate text-xs text-slate-500">
							{{ resolvedProfile.uri }}
						</div>
					</div>
					<div class="flex flex-wrap items-center gap-2">
						<NButton size="small" quaternary :disabled="!resolvedProfile" @click="resetToSaved">
							恢复
						</NButton>
						<NButton
							size="small"
							type="primary"
							:disabled="!resolvedProfile"
							:loading="isSaving"
							@click="save"
						>
							保存
						</NButton>
						<NButton size="small" quaternary @click="openVault">打开高级凭证库</NButton>
					</div>
				</div>

				<NAlert v-if="errorMessage" type="error" :bordered="false">
					{{ errorMessage }}
				</NAlert>

				<div class="rounded-md border border-slate-100 bg-slate-50/60 p-3">
					<div class="flex items-center justify-between gap-3">
						<div class="text-xs font-medium text-slate-700">认证配置</div>
						<NSwitch v-model:value="form.enabled" size="small" :disabled="!resolvedProfile" />
					</div>

					<div v-if="form.enabled" class="mt-3 space-y-3">
						<div class="space-y-1">
							<label class="text-xs text-slate-500">Provider</label>
							<NInput v-model:value="form.provider" placeholder="例如：s3 / gcs / azure" />
						</div>

						<div class="space-y-1">
							<label class="text-xs text-slate-500">Params (JSON)</label>
							<NInput
								v-model:value="form.paramsJson"
								type="textarea"
								:autosize="{ minRows: 3, maxRows: 10 }"
								placeholder='{"aws_access_key_id": "...", "aws_secret_access_key": "..."}'
							/>
							<div class="text-[11px] text-slate-400">
								提示：若使用 Stronghold 且希望沿用已选择的引用，可保持 params 为空。
							</div>
						</div>

						<div class="flex items-center justify-between text-xs text-slate-500">
							<span>保存到 Stronghold</span>
							<NSwitch v-model:value="form.saveToStronghold" size="small" />
						</div>

						<div v-if="form.saveToStronghold" class="space-y-2">
							<div class="space-y-1">
								<label class="text-xs text-slate-500">使用已有 Stronghold 凭证（可选）</label>
								<NSelect
									v-model:value="form.reference"
									filterable
									clearable
									:options="vaultOptions"
									placeholder="选择已有凭证引用（不选则保存时新建）"
								/>
								<div v-if="vaultLoadError" class="text-[11px] text-amber-600">
									{{ vaultLoadError }}
								</div>
							</div>

							<div v-if="form.reference" class="space-y-1">
								<label class="text-xs text-slate-500">Stronghold 引用</label>
								<NInput v-model:value="form.reference" readonly />
								<div class="text-[11px] text-slate-400">
									保存时：若填写 params，将覆盖该引用的内容；若 params 为空，将沿用该引用。
								</div>
							</div>

							<div v-if="selectedCredential" class="rounded-md bg-white p-2 text-xs text-slate-600">
								<div class="flex flex-wrap items-center justify-between gap-2">
									<div class="font-medium text-slate-700">已选择的凭证摘要</div>
									<div class="text-[11px] text-slate-400">更新时间：{{ selectedCredential.updatedAt }}</div>
								</div>
								<div class="mt-1">Provider：{{ selectedCredential.provider }}</div>
								<div class="mt-1">Label：{{ selectedCredential.label ?? "—" }}</div>
							</div>
						</div>

						<p v-else class="text-xs text-amber-500">
							关闭 Stronghold 将明文写入连接档案。
						</p>
					</div>
				</div>

				<div class="mt-3 rounded-md border border-slate-100 bg-white p-3">
					<div class="text-xs font-medium text-slate-700">当前生效</div>
					<div class="mt-1 text-sm text-slate-700">{{ savedAuthTitle }}</div>
					<div v-if="savedReference" class="mt-2 space-y-1">
						<div class="text-xs text-slate-500">reference</div>
						<div class="font-mono text-xs text-slate-600">{{ savedReference }}</div>
						<div class="text-[11px] text-slate-400">
							复用情况：{{ referencedByProfiles.length }} 个连接引用该凭证。
						</div>
						<div v-if="referencedByProfiles.length > 1" class="mt-1 flex flex-wrap gap-2">
							<NTag
								v-for="item in referencedByProfiles"
								:key="item.id"
								size="small"
								:type="item.id === resolvedProfile?.id ? 'info' : 'default'"
							>
								{{ item.name }}
							</NTag>
						</div>
					</div>
					<div class="mt-2 text-[11px] text-slate-400">
						建议在“连接”维度配置认证；Stronghold 全局凭证库仅用于查看与回收未引用凭证。
					</div>
				</div>
			</div>
		</NCard>

		<NEmpty v-if="!resolvedProfile" description="请先选择连接" />
	</div>
</template>

<script setup lang="ts">
import { emitTo } from "@tauri-apps/api/event"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
import { open } from "@tauri-apps/plugin-dialog"
import { FolderOpen } from "lucide-vue-next"
import { computed, onMounted, ref } from "vue"
import { useRoute } from "vue-router"

import type { AuthDescriptor } from "../ipc/v1"
import type { ConnectionKind } from "../lib/connectionKind"
import { getConnectionKind } from "../lib/connectionKind"
import { saveCredential } from "../lib/credentialVault"
import { normalizeConnectUri } from "../lib/lancedbUri"
import { loadProfileState } from "../stores/profiles"

const updateProfileEvent = "profiles:update"
const mainWindowLabel = "main"

const route = useRoute()

const form = ref({
	id: "",
	name: "",
	uri: "",
	storageOptionsJson: "{}",
})
const authForm = ref({
	enabled: false,
	provider: "",
	paramsJson: "{}",
	saveToStronghold: true,
	reference: "",
})
const errorMessage = ref("")
const isSubmitting = ref(false)
const isLoading = ref(true)

const editKind = computed<ConnectionKind>(() => {
	const value = form.value.uri.trim()
	if (!value) {
		return "local"
	}
	return getConnectionKind(value)
})

const uriPlaceholder = computed(() => {
	switch (editKind.value) {
		case "local":
			return "例如：E:\\data\\sample-db（数据库目录）"
		case "s3":
			return "例如：s3://bucket/path"
		case "remote":
			return "例如：db://host:port"
		case "gcs":
			return "例如：gs://bucket/path"
		case "azure":
			return "例如：az://container/path"
		default:
			return "请输入 URI"
	}
})

const showLocalPicker = computed(() => editKind.value === "local")

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

async function loadProfile() {
	errorMessage.value = ""
	const profileId = String(route.query.profileId ?? "").trim()
	if (!profileId) {
		errorMessage.value = "缺少连接档案 ID"
		isLoading.value = false
		return
	}

	try {
		const state = await loadProfileState()
		const profile = state.profiles.find((item) => item.id === profileId)
		if (!profile) {
			errorMessage.value = "连接档案不存在"
			return
		}
		form.value = {
			id: profile.id,
			name: profile.name ?? "",
			uri: profile.uri ?? "",
			storageOptionsJson: JSON.stringify(profile.storageOptions ?? {}, null, 2),
		}
		const auth = profile.auth ?? { type: "none" }
		if (auth.type === "none") {
			authForm.value = {
				enabled: false,
				provider: "",
				paramsJson: "{}",
				saveToStronghold: true,
				reference: "",
			}
		} else if (auth.type === "inline") {
			authForm.value = {
				enabled: true,
				provider: auth.provider,
				paramsJson: JSON.stringify(auth.params ?? {}, null, 2),
				saveToStronghold: false,
				reference: "",
			}
		} else {
			authForm.value = {
				enabled: true,
				provider: auth.provider,
				paramsJson: "{}",
				saveToStronghold: true,
				reference: auth.reference,
			}
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : "读取连接档案失败"
		errorMessage.value = message
	} finally {
		isLoading.value = false
	}
}

onMounted(() => {
	void loadProfile()
})

async function pickLocalFolder() {
	errorMessage.value = ""
	try {
		const selected = await open({
			directory: true,
			multiple: false,
			title: "选择 LanceDB 数据库目录",
		})

		if (!selected || Array.isArray(selected)) {
			return
		}

		form.value.uri = normalizeConnectUri(selected)
	} catch (error) {
		const message = error instanceof Error ? error.message : "打开文件夹选择器失败"
		errorMessage.value = message
	}
}

async function closeDialog() {
	try {
		await getCurrentWebviewWindow().close()
	} catch (error) {
		const message = error instanceof Error ? error.message : "关闭窗口失败"
		errorMessage.value = message
	}
}

async function saveProfile() {
	if (isSubmitting.value || isLoading.value) {
		return
	}

	errorMessage.value = ""
	const name = form.value.name.trim()
	const uri = form.value.uri.trim()
	if (!name || !uri) {
		errorMessage.value = "请填写连接名称与 URI"
		return
	}

	isSubmitting.value = true
	try {
		let auth: AuthDescriptor = { type: "none" }
		if (authForm.value.enabled) {
			const provider = authForm.value.provider.trim()
			if (!provider) {
				errorMessage.value = "请填写认证 Provider"
				return
			}
			const rawParams = authForm.value.paramsJson.trim()
			const paramsProvided = rawParams !== "" && rawParams !== "{}"
			const params = parseAuthParams(authForm.value.paramsJson)
			if (authForm.value.saveToStronghold) {
				if (!paramsProvided && authForm.value.reference.trim()) {
					auth = {
						type: "secret_ref",
						provider,
						reference: authForm.value.reference.trim(),
					}
				} else {
					const reference = await saveCredential({
						provider,
						params,
						label: name,
					})
					authForm.value.reference = reference
					auth = { type: "secret_ref", provider, reference }
				}
			} else {
				if (!paramsProvided && authForm.value.reference.trim()) {
					errorMessage.value = "请填写凭证参数，或保持 Stronghold 保存"
					return
				}
				auth = { type: "inline", provider, params }
			}
		}
		await emitTo(mainWindowLabel, updateProfileEvent, {
			id: form.value.id,
			name,
			uri,
			storageOptionsJson: form.value.storageOptionsJson?.trim()
				? form.value.storageOptionsJson
				: "{}",
			auth,
		})
		await closeDialog()
	} catch (error) {
		const message = error instanceof Error ? error.message : "更新连接失败"
		errorMessage.value = message
	} finally {
		isSubmitting.value = false
	}
}
</script>

<template>
	<div class="h-screen w-screen bg-slate-50 p-4">
		<NCard title="编辑连接" size="small" class="shadow-sm">
			<div class="space-y-3">
				<NAlert v-if="errorMessage" type="error" :bordered="false">
					{{ errorMessage }}
				</NAlert>
				<div v-if="isLoading" class="flex items-center gap-2 text-sm text-slate-500">
					<NSpin size="small" />
					<span>正在加载连接档案…</span>
				</div>
				<template v-else>
					<div class="space-y-1">
						<label class="text-xs text-slate-500">连接名称</label>
						<NInput v-model:value="form.name" placeholder="例如：本地样例库" />
					</div>
					<div class="space-y-1">
						<label class="text-xs text-slate-500">URI</label>
						<div class="flex items-center gap-2">
							<NInput
								v-model:value="form.uri"
								class="flex-1"
								:placeholder="uriPlaceholder"
							/>
							<NButton
								v-if="showLocalPicker"
								size="small"
								quaternary
								@click="pickLocalFolder"
							>
								<FolderOpen class="h-4 w-4" />
								<span class="ml-1">选择文件夹</span>
							</NButton>
						</div>
						<div v-if="showLocalPicker" class="text-xs text-slate-400">
							选择 LanceDB 的数据库根目录（例如 sample-db）。如果误选了 items.lance 这类 *.lance 目录，会自动改用它的上级目录。
						</div>
					</div>
					<div class="space-y-1">
						<label class="text-xs text-slate-500">storageOptions (JSON)</label>
						<NInput
							v-model:value="form.storageOptionsJson"
							type="textarea"
							:autosize="{ minRows: 4, maxRows: 10 }"
							placeholder='{"aws_region": "us-east-1"}'
						/>
					</div>

					<div class="space-y-2 rounded-md border border-slate-100 bg-slate-50/60 p-3">
						<div class="flex items-center justify-between">
							<label class="text-xs font-medium text-slate-600">Auth Descriptor</label>
							<NSwitch v-model:value="authForm.enabled" size="small" />
						</div>
						<div v-if="authForm.enabled" class="space-y-2">
							<div class="space-y-1">
								<label class="text-xs text-slate-500">Provider</label>
								<NInput
									v-model:value="authForm.provider"
									placeholder="例如：s3 / gcs / azure"
								/>
							</div>
							<div class="space-y-1">
								<label class="text-xs text-slate-500">Params (JSON)</label>
								<NInput
									v-model:value="authForm.paramsJson"
									type="textarea"
									:autosize="{ minRows: 3, maxRows: 8 }"
									placeholder='{"aws_access_key_id": "...", "aws_secret_access_key": "..."}'
								/>
							</div>
							<div class="flex items-center justify-between text-xs text-slate-500">
								<span>保存到 Stronghold</span>
								<NSwitch v-model:value="authForm.saveToStronghold" size="small" />
							</div>
							<div
								v-if="authForm.saveToStronghold && authForm.reference"
								class="space-y-1"
							>
								<label class="text-xs text-slate-500">Stronghold 引用</label>
								<NInput v-model:value="authForm.reference" readonly />
								<p class="text-xs text-slate-400">
									保持 params 为空将沿用当前引用。
								</p>
							</div>
							<p v-if="!authForm.saveToStronghold" class="text-xs text-amber-500">
								关闭 Stronghold 将明文写入连接档案。
							</p>
						</div>
					</div>

					<div class="flex items-center justify-end gap-2 pt-2">
						<NButton size="small" quaternary :disabled="isSubmitting" @click="closeDialog">
							取消
						</NButton>
						<NButton
							size="small"
							type="primary"
							:loading="isSubmitting"
							@click="saveProfile"
						>
							保存
						</NButton>
					</div>
				</template>
			</div>
		</NCard>
	</div>
</template>

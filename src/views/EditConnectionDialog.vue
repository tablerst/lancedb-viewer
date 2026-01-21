<script setup lang="ts">
import { emitTo } from "@tauri-apps/api/event"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
import { open } from "@tauri-apps/plugin-dialog"
import { FolderOpen } from "lucide-vue-next"
import { computed, onMounted, ref } from "vue"
import { useRoute } from "vue-router"

import type { ConnectionKind } from "../lib/connectionKind"
import { getConnectionKind } from "../lib/connectionKind"
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
		await emitTo(mainWindowLabel, updateProfileEvent, {
			id: form.value.id,
			name,
			uri,
			storageOptionsJson: form.value.storageOptionsJson?.trim()
				? form.value.storageOptionsJson
				: "{}",
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

import { computed, onMounted, ref } from "vue"

import { cleanupUnusedCredentials } from "../lib/credentialVault"
import { getConnectionKind } from "../lib/connectionKind"
import { collectCredentialReferences } from "../lib/credentialReferences"
import { normalizeConnectUri } from "../lib/lancedbUri"
import type { AuthDescriptor } from "../ipc/v1"
import type { StoredProfile } from "../models/profile"
import { createProfile, loadProfileState, saveProfileState } from "../stores/profiles"

interface ProfileFormState {
	name: string
	uri: string
	storageOptionsJson: string
	auth?: AuthDescriptor
}

interface UseProfilesOptions {
	onStatus?: (message: string) => void
	onError?: (message: string) => void
}

function parseStorageOptions(raw: string): Record<string, string> {
	if (!raw.trim()) {
		return {}
	}
	const parsed = JSON.parse(raw) as Record<string, unknown>
	if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") {
		throw new Error("storageOptions 必须是 JSON 对象")
	}
	return Object.fromEntries(Object.entries(parsed).map(([key, value]) => [key, String(value)]))
}

export function useProfiles(options: UseProfilesOptions = {}) {
	const profiles = ref<StoredProfile[]>([])
	const activeProfileId = ref<string | null>(null)
	const profileForm = ref<ProfileFormState>({
		name: "",
		uri: "",
		storageOptionsJson: "{}",
		auth: { type: "none" },
	})
	const isSavingProfile = ref(false)

	const activeProfile = computed(
		() => profiles.value.find((profile) => profile.id === activeProfileId.value) ?? null
	)

	onMounted(async () => {
		const state = await loadProfileState()
		profiles.value = state.profiles
		activeProfileId.value = state.activeProfileId
	})

	async function persistProfiles() {
		await saveProfileState({
			profiles: profiles.value,
			activeProfileId: activeProfileId.value,
		})
	}

	async function cleanupCredentials(nextProfiles: StoredProfile[]) {
		try {
			const used = collectCredentialReferences(nextProfiles)
			const removed = await cleanupUnusedCredentials(used)
			if (removed.length > 0) {
				options.onStatus?.(`已回收 ${removed.length} 个未引用凭证`)
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : "清理凭证失败"
			options.onError?.(message)
		}
	}

	async function addProfile() {
		if (isSavingProfile.value) {
			return
		}

		const name = profileForm.value.name.trim()
		const uriRaw = profileForm.value.uri.trim()
		if (!name || !uriRaw) {
			options.onError?.("请填写连接名称与 URI")
			return
		}

		try {
			isSavingProfile.value = true
			const normalizedUri = normalizeConnectUri(uriRaw)
			if (!normalizedUri.trim()) {
				options.onError?.("URI 无效")
				return
			}
			if (getConnectionKind(normalizedUri) === "local" && uriRaw !== normalizedUri) {
				options.onStatus?.("已规范化本地路径（例如移除 file:// 或将 *.lance 转为数据库目录）")
			}
			const storageOptions = parseStorageOptions(profileForm.value.storageOptionsJson)
			const profile = createProfile({
				name,
				uri: normalizedUri,
				storageOptions,
				auth: profileForm.value.auth ?? { type: "none" },
			})
			profiles.value = [...profiles.value, profile]
			activeProfileId.value = profile.id
			await persistProfiles()
			profileForm.value = {
				name: "",
				uri: "",
				storageOptionsJson: "{}",
				auth: { type: "none" },
			}
			options.onStatus?.("连接档案已保存")
		} catch (error) {
			const message = error instanceof Error ? error.message : "解析 storageOptions 失败"
			options.onError?.(message)
		} finally {
			isSavingProfile.value = false
		}
	}

	async function updateProfile(input: {
		id: string
		name: string
		uri: string
		storageOptionsJson: string
		auth?: AuthDescriptor
	}) {
		if (isSavingProfile.value) {
			return
		}

		const name = input.name.trim()
		const uriRaw = input.uri.trim()
		if (!name || !uriRaw) {
			options.onError?.("请填写连接名称与 URI")
			return
		}

		const existing = profiles.value.find((profile) => profile.id === input.id)
		if (!existing) {
			options.onError?.("连接档案不存在")
			return
		}

		try {
			isSavingProfile.value = true
			const normalizedUri = normalizeConnectUri(uriRaw)
			if (!normalizedUri.trim()) {
				options.onError?.("URI 无效")
				return
			}
			if (getConnectionKind(normalizedUri) === "local" && uriRaw !== normalizedUri) {
				options.onStatus?.("已规范化本地路径（例如移除 file:// 或将 *.lance 转为数据库目录）")
			}
			const storageOptions = parseStorageOptions(input.storageOptionsJson)
			const updated: StoredProfile = {
				...existing,
				name,
				uri: normalizedUri,
				storageOptions,
				auth: input.auth ?? existing.auth ?? { type: "none" },
			}
			profiles.value = profiles.value.map((profile) =>
				profile.id === input.id ? updated : profile
			)
			await persistProfiles()
			options.onStatus?.("连接档案已更新")
			await cleanupCredentials(profiles.value)
		} catch (error) {
			const message = error instanceof Error ? error.message : "更新连接档案失败"
			options.onError?.(message)
		} finally {
			isSavingProfile.value = false
		}
	}

	async function deleteProfile(profileId: string) {
		if (isSavingProfile.value) {
			return
		}

		const existing = profiles.value.find((profile) => profile.id === profileId)
		if (!existing) {
			options.onError?.("连接档案不存在")
			return
		}

		try {
			isSavingProfile.value = true
			const nextProfiles = profiles.value.filter((profile) => profile.id !== profileId)
			profiles.value = nextProfiles
			if (activeProfileId.value === profileId) {
				activeProfileId.value = nextProfiles[0]?.id ?? null
			}
			await persistProfiles()
			options.onStatus?.("连接档案已删除")
			await cleanupCredentials(profiles.value)
		} catch (error) {
			const message = error instanceof Error ? error.message : "删除连接档案失败"
			options.onError?.(message)
		} finally {
			isSavingProfile.value = false
		}
	}

	async function setProfileLastConnected(profileId: string, connectedAt: string) {
		const existing = profiles.value.find((profile) => profile.id === profileId)
		if (!existing) {
			return
		}
		profiles.value = profiles.value.map((profile) =>
			profile.id === profileId ? { ...profile, lastConnectedAt: connectedAt } : profile
		)
		await persistProfiles()
	}

	async function selectProfile(profileId: string) {
		activeProfileId.value = profileId
		await persistProfiles()
	}

	return {
		profiles,
		activeProfileId,
		activeProfile,
		profileForm,
		isSavingProfile,
		addProfile,
		updateProfile,
		deleteProfile,
		setProfileLastConnected,
		selectProfile,
	}
}

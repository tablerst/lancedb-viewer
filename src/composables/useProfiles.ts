import { computed, onMounted, ref } from "vue";

import type { StoredProfile } from "../models/profile";
import { createProfile, loadProfileState, saveProfileState } from "../stores/profiles";

interface ProfileFormState {
	name: string;
	uri: string;
	storageOptionsJson: string;
}

interface UseProfilesOptions {
	onStatus?: (message: string) => void;
	onError?: (message: string) => void;
}

function parseStorageOptions(raw: string): Record<string, string> {
	if (!raw.trim()) {
		return {};
	}
	const parsed = JSON.parse(raw) as Record<string, unknown>;
	if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") {
		throw new Error("storageOptions 必须是 JSON 对象");
	}
	return Object.fromEntries(
		Object.entries(parsed).map(([key, value]) => [key, String(value)]),
	);
}

export function useProfiles(options: UseProfilesOptions = {}) {
	const profiles = ref<StoredProfile[]>([]);
	const activeProfileId = ref<string | null>(null);
	const profileForm = ref<ProfileFormState>({
		name: "",
		uri: "",
		storageOptionsJson: "{}",
	});
	const isSavingProfile = ref(false);

	const activeProfile = computed(
		() => profiles.value.find((profile) => profile.id === activeProfileId.value) ?? null,
	);

	onMounted(async () => {
		const state = await loadProfileState();
		profiles.value = state.profiles;
		activeProfileId.value = state.activeProfileId;
	});

	async function persistProfiles() {
		await saveProfileState({
			profiles: profiles.value,
			activeProfileId: activeProfileId.value,
		});
	}

	async function addProfile() {
		if (isSavingProfile.value) {
			return;
		}

		const name = profileForm.value.name.trim();
		const uri = profileForm.value.uri.trim();
		if (!name || !uri) {
			options.onError?.("请填写连接名称与 URI");
			return;
		}

		try {
			isSavingProfile.value = true;
			const storageOptions = parseStorageOptions(profileForm.value.storageOptionsJson);
			const profile = createProfile({
				name,
				uri,
				storageOptions,
				auth: { type: "none" },
			});
			profiles.value = [...profiles.value, profile];
			activeProfileId.value = profile.id;
			await persistProfiles();
			profileForm.value = { name: "", uri: "", storageOptionsJson: "{}" };
			options.onStatus?.("连接档案已保存");
		} catch (error) {
			const message = error instanceof Error ? error.message : "解析 storageOptions 失败";
			options.onError?.(message);
		} finally {
			isSavingProfile.value = false;
		}
	}

	async function selectProfile(profileId: string) {
		activeProfileId.value = profileId;
		await persistProfiles();
	}

	return {
		profiles,
		activeProfileId,
		activeProfile,
		profileForm,
		isSavingProfile,
		addProfile,
		selectProfile,
	};
}

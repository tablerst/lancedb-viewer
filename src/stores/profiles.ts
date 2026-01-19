import { LazyStore } from "@tauri-apps/plugin-store";

import type { NewProfileInput, ProfileState, StoredProfile } from "../models/profile";

const store = new LazyStore("profiles.json");

const emptyState: ProfileState = {
	profiles: [],
	activeProfileId: null,
};

export async function loadProfileState(): Promise<ProfileState> {
	const stored = await store.get<ProfileState>("profiles");
	if (!stored) {
		return { ...emptyState };
	}
	return {
		profiles: stored.profiles ?? [],
		activeProfileId: stored.activeProfileId ?? null,
	};
}

export async function saveProfileState(state: ProfileState): Promise<void> {
	await store.set("profiles", state);
	await store.save();
}

export function createProfile(input: NewProfileInput): StoredProfile {
	return {
		id: crypto.randomUUID(),
		name: input.name,
		uri: input.uri,
		storageOptions: input.storageOptions ?? {},
		options: input.options,
		auth: input.auth,
	};
}

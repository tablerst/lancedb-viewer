import type { AuthDescriptor, ConnectOptions, ConnectProfile } from "../ipc/v1"
import { normalizeConnectUri } from "../lib/lancedbUri"

export interface StoredProfile {
	id: string
	name: string
	uri: string
	storageOptions: Record<string, string>
	options?: ConnectOptions
	auth?: AuthDescriptor
}

export interface ProfileState {
	profiles: StoredProfile[]
	activeProfileId: string | null
}

export interface NewProfileInput {
	name: string
	uri: string
	storageOptions?: Record<string, string>
	options?: ConnectOptions
	auth?: AuthDescriptor
}

export function toConnectProfile(profile: StoredProfile): ConnectProfile {
	return {
		name: profile.name,
		uri: normalizeConnectUri(profile.uri),
		storageOptions: profile.storageOptions,
		options: profile.options,
		auth: profile.auth,
	}
}

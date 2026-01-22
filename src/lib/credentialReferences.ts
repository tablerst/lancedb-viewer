import type { StoredProfile } from "../models/profile"

export function collectCredentialReferences(profiles: StoredProfile[]): Set<string> {
	const references = new Set<string>()
	for (const profile of profiles) {
		const auth = profile.auth
		if (auth?.type !== "secret_ref") {
			continue
		}
		const reference = auth.reference.trim()
		if (reference) {
			references.add(reference)
		}
	}
	return references
}

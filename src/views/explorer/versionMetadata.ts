export type MetadataEntry = {
	key: string
	value: string
}

export function getMetadataEntries(metadata: Record<string, string>): MetadataEntry[] {
	return Object.entries(metadata ?? {}).map(([key, value]) => ({ key, value }))
}

export function formatMetadata(metadata: Record<string, string>) {
	const entries = getMetadataEntries(metadata)
	if (!entries.length) {
		return "—"
	}
	return entries.map(({ key, value }) => `${key}=${value}`).join(", ")
}

import { LazyStore } from "@tauri-apps/plugin-store"

export interface CredentialSummary {
	reference: string
	provider: string
	label?: string
	updatedAt: string
}

export interface CredentialRecord extends CredentialSummary {
	params: Record<string, string>
}

type StrongholdHandle = {
	setItem: (key: string, value: Uint8Array) => Promise<void>
	getItem: (key: string) => Promise<unknown>
	deleteItem: (key: string) => Promise<void>
	clear: () => Promise<void>
	save: () => Promise<void>
}

type StrongholdConstructor = {
	load: (vault: string, passphrase: string) => Promise<StrongholdHandle>
}

type StrongholdModule = {
	Stronghold: StrongholdConstructor
}

const metaStore = new LazyStore("credentials-meta.json")
const configStore = new LazyStore("credentials-config.json")

const indexKey = "credentials"
const passphraseKey = "strongholdPassphrase"
const vaultName = "lancedb-viewer"

const encoder = new TextEncoder()
const decoder = new TextDecoder()

let strongholdPromise: Promise<StrongholdHandle> | null = null

async function loadStrongholdModule(): Promise<StrongholdModule> {
	const module = (await import("@tauri-apps/plugin-stronghold")) as unknown
	if (typeof module !== "object" || module === null) {
		throw new Error("Stronghold 插件未正确加载")
	}
	const candidates = module as { Stronghold?: unknown; stronghold?: unknown }
	const stronghold = candidates.Stronghold ?? candidates.stronghold
	if (!stronghold || typeof (stronghold as { load?: unknown }).load !== "function") {
		throw new Error("Stronghold API 不兼容")
	}
	return { Stronghold: stronghold as StrongholdConstructor }
}

async function getPassphrase(): Promise<string> {
	const stored = await configStore.get<string>(passphraseKey)
	if (typeof stored === "string" && stored.trim()) {
		return stored
	}
	const generated = crypto.randomUUID()
	await configStore.set(passphraseKey, generated)
	await configStore.save()
	return generated
}

async function getStronghold(): Promise<StrongholdHandle> {
	if (!strongholdPromise) {
		strongholdPromise = (async () => {
			const { Stronghold } = await loadStrongholdModule()
			const passphrase = await getPassphrase()
			return Stronghold.load(vaultName, passphrase)
		})()
	}
	return strongholdPromise
}

function decodePayload(value: unknown): string {
	if (typeof value === "string") {
		return value
	}
	if (value instanceof Uint8Array) {
		return decoder.decode(value)
	}
	throw new Error("Stronghold 返回值格式不支持")
}

function normalizeParams(input: Record<string, unknown>): Record<string, string> {
	return Object.fromEntries(
		Object.entries(input).map(([key, value]) => [key, String(value)])
	)
}

async function loadIndex(): Promise<CredentialSummary[]> {
	const stored = await metaStore.get<CredentialSummary[]>(indexKey)
	if (!stored) {
		return []
	}
	return stored
}

async function saveIndex(items: CredentialSummary[]): Promise<void> {
	await metaStore.set(indexKey, items)
	await metaStore.save()
}

export async function saveCredential(input: {
	provider: string
	params: Record<string, string>
	label?: string
	reference?: string
}): Promise<string> {
	const stronghold = await getStronghold()
	const reference = input.reference?.trim() || `cred_${crypto.randomUUID()}`
	const payload = JSON.stringify({ params: input.params })
	await stronghold.setItem(reference, encoder.encode(payload))
	await stronghold.save()

	const updatedAt = new Date().toISOString()
	const index = await loadIndex()
	const existingIndex = index.findIndex((item) => item.reference === reference)
	const nextItem: CredentialSummary = {
		reference,
		provider: input.provider,
		label: input.label,
		updatedAt,
	}
	if (existingIndex >= 0) {
		index[existingIndex] = nextItem
	} else {
		index.push(nextItem)
	}
	await saveIndex(index)
	return reference
}

export async function getCredential(reference: string): Promise<CredentialRecord | null> {
	const stronghold = await getStronghold()
	const payload = await stronghold.getItem(reference)
	if (!payload) {
		return null
	}
	const decoded = decodePayload(payload)
	const parsed = JSON.parse(decoded) as { params?: Record<string, unknown> }
	const params = normalizeParams(parsed.params ?? {})
	const index = await loadIndex()
	const meta = index.find((item) => item.reference === reference)

	return {
		reference,
		provider: meta?.provider ?? "unknown",
		label: meta?.label,
		updatedAt: meta?.updatedAt ?? new Date().toISOString(),
		params,
	}
}

export async function listCredentials(): Promise<CredentialSummary[]> {
	return loadIndex()
}

export async function deleteCredential(reference: string): Promise<void> {
	const stronghold = await getStronghold()
	await stronghold.deleteItem(reference)
	await stronghold.save()
	const index = await loadIndex()
	const next = index.filter((item) => item.reference !== reference)
	await saveIndex(next)
}

export async function clearCredentials(): Promise<void> {
	const stronghold = await getStronghold()
	await stronghold.clear()
	await stronghold.save()
	await saveIndex([])
}

export async function cleanupUnusedCredentials(
	usedReferences: Set<string>
): Promise<string[]> {
	const removed: string[] = []
	const credentials = await loadIndex()
	for (const credential of credentials) {
		if (usedReferences.has(credential.reference)) {
			continue
		}
		await deleteCredential(credential.reference)
		removed.push(credential.reference)
	}
	return removed
}

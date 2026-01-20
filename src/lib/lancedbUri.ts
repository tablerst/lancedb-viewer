function stripWrappingQuotes(value: string): string {
	const trimmed = value.trim()
	if (trimmed.length >= 2) {
		const first = trimmed[0]
		const last = trimmed[trimmed.length - 1]
		if ((first === '"' && last === '"') || (first === "'" && last === "'")) {
			return trimmed.slice(1, -1)
		}
	}
	return trimmed
}

function stripFileScheme(uri: string): string {
	const lower = uri.toLowerCase()
	if (!lower.startsWith("file://")) {
		return uri
	}

	// Common forms:
	// - file:///C:/path
	// - file://C:/path
	// - file:///home/user/path
	let without = uri.slice("file://".length)
	// If it looks like /C:/... (Windows file URI), drop the leading slash.
	if (/^\/[a-zA-Z]:\//.test(without)) {
		without = without.slice(1)
	}
	return without
}

function trimTrailingSeparators(pathLike: string): string {
	return pathLike.replace(/[\\/]+$/u, "")
}

function parentDirectory(pathLike: string): string {
	const normalized = trimTrailingSeparators(pathLike)
	const lastSlash = Math.max(normalized.lastIndexOf("/"), normalized.lastIndexOf("\\"))
	if (lastSlash <= 0) {
		return normalized
	}
	return normalized.slice(0, lastSlash)
}

function endsWithLanceTableDir(pathLike: string): boolean {
	return trimTrailingSeparators(pathLike).toLowerCase().endsWith(".lance")
}

/**
 * Normalizes a URI before sending it to the Rust backend.
 *
 * Notes:
 * - LanceDB local connections expect a directory path (no scheme).
 * - Users often paste `file://...` or accidentally pick a `*.lance` table directory.
 */
export function normalizeConnectUri(raw: string): string {
	let uri = stripWrappingQuotes(raw)
	uri = stripFileScheme(uri)
	uri = uri.trim()

	// If user selected a single table directory (e.g. items.lance), use its parent as DB root.
	// This makes the UI forgiving and aligns with how LanceDB expects a database directory.
	if (!uri.toLowerCase().includes("://") && endsWithLanceTableDir(uri)) {
		uri = parentDirectory(uri)
	}

	return uri
}

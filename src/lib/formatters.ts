function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value)
}

export function normalizeRow(row: unknown): Record<string, unknown> {
	if (isRecord(row)) {
		return row
	}
	return { value: row }
}

export function formatCellValue(value: unknown): string {
	if (value === null || value === undefined) {
		return ""
	}

	if (typeof value === "string") {
		return value
	}

	if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
		return String(value)
	}

	if (value instanceof Date) {
		return value.toISOString()
	}

	if (value instanceof Uint8Array) {
		return `Uint8Array(${value.length})`
	}

	if (Array.isArray(value)) {
		const preview = value.slice(0, 6).map(formatCellValue).join(", ")
		const suffix = value.length > 6 ? " …" : ""
		return `[${preview}${suffix}]`
	}

	if (typeof value === "object") {
		try {
			return JSON.stringify(value)
		} catch {
			return "[Object]"
		}
	}

	return String(value)
}

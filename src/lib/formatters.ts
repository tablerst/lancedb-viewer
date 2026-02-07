import { h, type VNodeChild } from "vue"

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

/**
 * Rich cell renderer – returns VNodes for styled display of special types.
 * Used in NDataTable `render` functions for better visual presentation.
 */
export function renderCellValue(value: unknown): VNodeChild {
	if (value === null || value === undefined) {
		return h("span", { class: "text-slate-300 italic select-none" }, "NULL")
	}

	if (typeof value === "boolean") {
		return h("span", { class: "select-none" }, value ? "✅" : "❌")
	}

	if (Array.isArray(value)) {
		const dim = value.length
		const previewCount = 4
		const preview = value
			.slice(0, previewCount)
			.map((v) => formatCellValue(v))
			.join(", ")
		const suffix = dim > previewCount ? ", …" : ""
		return h("span", { class: "font-mono text-xs" }, [
			h("span", { class: "mr-1 text-slate-400" }, `[${dim}d]`),
			`${preview}${suffix}`,
		])
	}

	if (value instanceof Uint8Array) {
		return h("span", { class: "font-mono text-xs text-slate-400" }, `Binary(${value.length})`)
	}

	if (typeof value === "object") {
		try {
			const json = JSON.stringify(value)
			return h("span", { class: "font-mono text-xs" }, json)
		} catch {
			return h("span", { class: "text-slate-400 italic" }, "[Object]")
		}
	}

	return formatCellValue(value)
}

export function formatTimestamp(value?: string | null): string {
	if (!value) {
		return "尚未连接"
	}
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) {
		return "尚未连接"
	}
	return new Intl.DateTimeFormat("zh-CN", {
		year: "numeric",
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
	}).format(date)
}

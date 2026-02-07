import type { SelectOption } from "naive-ui"
import type { InjectionKey, Ref } from "vue"
import { h } from "vue"
import type { FieldDataType, IndexTypeV1, SchemaFieldInput } from "../../ipc/v1"

// ── Injection Keys ─────────────────────────────────────

/** Reactive counter; incremented when data should be re-scanned. */
export const DATA_REFRESH_KEY: InjectionKey<Readonly<Ref<number>>> = Symbol("dataRefresh")

/** Call this to request a data refresh (bumps the counter). */
export const TRIGGER_DATA_REFRESH_KEY: InjectionKey<() => void> = Symbol("triggerDataRefresh")

// ── Types ──────────────────────────────────────────────

export type FieldDraft = {
	name: string
	dataType: FieldDataType
	nullable: boolean
	vectorLength?: number
}

export type AlterDraft = {
	path: string
	rename: string
	nullable: "keep" | "nullable" | "not_nullable"
	dataType: "keep" | FieldDataType
	vectorLength?: number
}

export type UpdateDraft = {
	column: string
	expr: string
}

export type ColumnOpsTab = "add" | "alter" | "drop"

// ── Select Options ─────────────────────────────────────

export const fieldTypeOptions: SelectOption[] = [
	{ label: "Int8", value: "int8" },
	{ label: "Int16", value: "int16" },
	{ label: "Int32", value: "int32" },
	{ label: "Int64", value: "int64" },
	{ label: "UInt8", value: "uint8" },
	{ label: "UInt16", value: "uint16" },
	{ label: "UInt32", value: "uint32" },
	{ label: "UInt64", value: "uint64" },
	{ label: "Float32", value: "float32" },
	{ label: "Float64", value: "float64" },
	{ label: "Boolean", value: "boolean" },
	{ label: "Utf8", value: "utf8" },
	{ label: "LargeUtf8", value: "large_utf8" },
	{ label: "Binary", value: "binary" },
	{ label: "LargeBinary", value: "large_binary" },
	{ label: "Vector(Float32)", value: "fixed_size_list_float32" },
]

export const alterTypeOptions: SelectOption[] = [
	{ label: "保持不变", value: "keep" },
	...fieldTypeOptions,
]

export const nullableOptions: SelectOption[] = [
	{ label: "保持不变", value: "keep" },
	{ label: "可为空", value: "nullable" },
	{ label: "不可为空", value: "not_nullable" },
]

export const indexTypeLabels: Record<IndexTypeV1, string> = {
	auto: "Auto",
	btree: "BTree",
	bitmap: "Bitmap",
	label_list: "LabelList",
	fts: "FTS",
	ivf_flat: "IVF_FLAT",
	ivf_sq: "IVF_SQ",
	ivf_pq: "IVF_PQ",
	ivf_rq: "IVF_RQ",
	ivf_hnsw_pq: "IVF_HNSW_PQ",
	ivf_hnsw_sq: "IVF_HNSW_SQ",
}

export const indexTypeOptions: SelectOption[] = [
	{ label: "Auto", value: "auto" },
	{ label: "BTree", value: "btree" },
	{ label: "Bitmap", value: "bitmap" },
	{ label: "LabelList", value: "label_list" },
	{ label: "FTS", value: "fts" },
	{ label: "IVF_FLAT", value: "ivf_flat" },
	{ label: "IVF_SQ", value: "ivf_sq" },
	{ label: "IVF_PQ", value: "ivf_pq" },
	{ label: "IVF_RQ", value: "ivf_rq" },
	{ label: "IVF_HNSW_PQ", value: "ivf_hnsw_pq" },
	{ label: "IVF_HNSW_SQ", value: "ivf_hnsw_sq" },
]

export const writeModeOptions: SelectOption[] = [
	{ label: "追加写入", value: "append" },
	{ label: "覆盖写入", value: "overwrite" },
]

export const fileFormatOptions: SelectOption[] = [
	{ label: "CSV", value: "csv" },
	{ label: "Parquet", value: "parquet" },
	{ label: "JSONL", value: "jsonl" },
]

// ── Utility Functions ──────────────────────────────────

export function renderHeader(title: string) {
	return h("span", { class: "table-header-ellipsis", title }, title)
}

export function formatMetadata(metadata: Record<string, string>) {
	const entries = Object.entries(metadata ?? {})
	if (!entries.length) {
		return "—"
	}
	return entries.map(([key, value]) => `${key}=${value}`).join(", ")
}

export function createFieldDraft(): FieldDraft {
	return { name: "", dataType: "utf8", nullable: true, vectorLength: 3 }
}

export function createAlterDraft(): AlterDraft {
	return { path: "", rename: "", nullable: "keep", dataType: "keep", vectorLength: 3 }
}

export function isVectorType(value: FieldDataType | "keep") {
	return value === "fixed_size_list_float32"
}

export function toFieldInput(draft: FieldDraft): SchemaFieldInput | null {
	const name = draft.name.trim()
	if (!name) {
		return null
	}
	const input: SchemaFieldInput = { name, dataType: draft.dataType, nullable: draft.nullable }
	if (isVectorType(draft.dataType)) {
		const length = Number(draft.vectorLength ?? 0)
		if (Number.isFinite(length) && length > 0) {
			input.vectorLength = length
		}
	}
	return input
}

export function resolveNullable(value: AlterDraft["nullable"]): boolean | undefined {
	if (value === "keep") {
		return undefined
	}
	return value === "nullable"
}

export function compareValues(a: unknown, b: unknown) {
	if (a === b) {
		return 0
	}
	if (a === null || a === undefined) {
		return -1
	}
	if (b === null || b === undefined) {
		return 1
	}
	const numA = typeof a === "number" ? a : Number(a)
	const numB = typeof b === "number" ? b : Number(b)
	if (!Number.isNaN(numA) && !Number.isNaN(numB)) {
		return numA - numB
	}
	return String(a).localeCompare(String(b))
}

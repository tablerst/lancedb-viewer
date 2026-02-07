/**
 * Parses user-entered column filter expressions into SQL-like filter strings
 * compatible with the LanceDB scan API.
 */

const NUMERIC_TYPES = new Set([
	"int8",
	"int16",
	"int32",
	"int64",
	"uint8",
	"uint16",
	"uint32",
	"uint64",
	"float16",
	"float32",
	"float64",
])

/** Check if a dataType string represents a numeric type (case-insensitive). */
export function isNumericType(dataType?: string): boolean {
	if (!dataType) return false
	return NUMERIC_TYPES.has(dataType.toLowerCase())
}

/** Check if a dataType string represents a vector/array type. */
export function isVectorOrArrayType(dataType?: string): boolean {
	if (!dataType) return false
	const lower = dataType.toLowerCase()
	return lower.startsWith("fixedsizelist") || lower.startsWith("list") || lower.startsWith("[")
}

/** Check if a dataType string represents a binary type. */
export function isBinaryType(dataType?: string): boolean {
	if (!dataType) return false
	const lower = dataType.toLowerCase()
	return lower === "binary" || lower === "largebinary" || lower === "large_binary"
}

/** Check if a column is editable based on its data type. */
export function isEditableType(dataType?: string): boolean {
	if (!dataType) return false
	if (isVectorOrArrayType(dataType)) return false
	if (isBinaryType(dataType)) return false
	return true
}

/**
 * Parse a single column filter expression into a SQL-like predicate.
 *
 * Supported syntax:
 * - `NULL` → `column IS NULL`
 * - `NOT NULL` → `column IS NOT NULL`
 * - `1..100` → `column >= 1 AND column <= 100`
 * - `> 5`, `>= 10`, `< 3`, `<= 7`, `!= "abc"`, `= 42` → operator expressions
 * - Plain text → `column = value` (numeric) or `column LIKE '%value%'` (string)
 */
export function parseColumnFilter(column: string, value: string, dataType?: string): string | null {
	const trimmed = value.trim()
	if (!trimmed) return null

	const upper = trimmed.toUpperCase()

	// NULL / NOT NULL
	if (upper === "NULL") return `${column} IS NULL`
	if (upper === "NOT NULL" || upper === "!NULL") return `${column} IS NOT NULL`

	// Range: "1..100"
	const rangeMatch = trimmed.match(/^(-?\d+(?:\.\d+)?)\s*\.\.\s*(-?\d+(?:\.\d+)?)$/)
	if (rangeMatch) {
		return `${column} >= ${rangeMatch[1]} AND ${column} <= ${rangeMatch[2]}`
	}

	// Operator prefix: >=, <=, !=, >, <, =
	const opMatch = trimmed.match(/^(>=|<=|!=|<>|>|<|=)\s*(.+)$/)
	if (opMatch) {
		const [, op, val] = opMatch
		const normalizedOp = op === "<>" ? "!=" : op
		const rawVal = val.trim()
		if (isNumericType(dataType)) {
			const num = Number(rawVal)
			if (!Number.isNaN(num)) {
				return `${column} ${normalizedOp} ${num}`
			}
		}
		// Remove surrounding quotes if present
		const unquoted = rawVal.replace(/^["']|["']$/g, "")
		return `${column} ${normalizedOp} '${unquoted}'`
	}

	// Default: numeric → equality, string → LIKE
	if (isNumericType(dataType)) {
		const num = Number(trimmed)
		if (!Number.isNaN(num)) {
			return `${column} = ${num}`
		}
		// Not a valid number, try LIKE on cast
		return `CAST(${column} AS TEXT) LIKE '%${trimmed}%'`
	}

	return `${column} LIKE '%${trimmed}%'`
}

/**
 * Build a combined filter expression from column filters and a global filter.
 */
export function buildFilterExpression(
	columnFilters: Record<string, string>,
	globalFilter: string,
	schemaFields?: Array<{ name: string; dataType: string }>
): string | undefined {
	const conditions: string[] = []

	for (const [column, rawValue] of Object.entries(columnFilters)) {
		const field = schemaFields?.find((f) => f.name === column)
		const condition = parseColumnFilter(column, rawValue, field?.dataType)
		if (condition) conditions.push(condition)
	}

	const global = globalFilter.trim()
	if (global) {
		conditions.push(global)
	}

	return conditions.length ? conditions.join(" AND ") : undefined
}

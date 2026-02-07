import { computed, ref } from "vue"
import type { CellAddress, DataGridChangeset, DataGridColumn, RowStatus } from "./types"

/**
 * Manages inline editing state for the DataGrid:
 * dirty tracking, new rows, pending deletes, and changeset generation.
 */
export function useDataGridEditing(
	columns: () => DataGridColumn[],
	rows: () => Record<string, unknown>[]
) {
	/** Currently active (editing) cell. Null when not editing. */
	const activeCell = ref<CellAddress | null>(null)

	/** Dirty values: rowIndex → { columnKey → newValue } */
	const dirtyRows = ref<Map<number, Map<string, unknown>>>(new Map())

	/** Newly inserted rows (not yet saved). */
	const newRows = ref<Record<string, unknown>[]>([])

	/** Row indices marked for deletion. */
	const pendingDeletes = ref<Set<number>>(new Set())

	const hasChanges = computed(
		() => dirtyRows.value.size > 0 || newRows.value.length > 0 || pendingDeletes.value.size > 0
	)

	const dirtyCount = computed(
		() => dirtyRows.value.size + newRows.value.length + pendingDeletes.value.size
	)

	// ── Cell editing ───────────────────────────────────

	function startEditing(rowIndex: number, columnKey: string) {
		const col = columns().find((c) => c.key === columnKey)
		if (!col || col.editable === false) return
		activeCell.value = { rowIndex, columnKey }
	}

	function stopEditing() {
		activeCell.value = null
	}

	function isEditing(rowIndex: number, columnKey: string): boolean {
		const ac = activeCell.value
		return ac !== null && ac.rowIndex === rowIndex && ac.columnKey === columnKey
	}

	function commitCellEdit(rowIndex: number, columnKey: string, newValue: unknown) {
		const currentRows = rows()
		// For new rows (index beyond currentRows), handle separately
		const newRowStartIndex = currentRows.length
		if (rowIndex >= newRowStartIndex) {
			const newRowIdx = rowIndex - newRowStartIndex
			if (newRowIdx < newRows.value.length) {
				newRows.value[newRowIdx] = { ...newRows.value[newRowIdx], [columnKey]: newValue }
			}
			stopEditing()
			return
		}

		const originalRow = currentRows[rowIndex]
		if (!originalRow) {
			stopEditing()
			return
		}

		// Check if value actually changed
		const originalValue = originalRow[columnKey]
		if (originalValue === newValue) {
			stopEditing()
			return
		}

		const rowDirty = dirtyRows.value.get(rowIndex) ?? new Map<string, unknown>()
		rowDirty.set(columnKey, newValue)
		dirtyRows.value = new Map(dirtyRows.value).set(rowIndex, rowDirty)
		stopEditing()
	}

	function cancelCellEdit() {
		stopEditing()
	}

	/** Revert all dirty changes for a row. */
	function revertRow(rowIndex: number) {
		const next = new Map(dirtyRows.value)
		next.delete(rowIndex)
		dirtyRows.value = next
		pendingDeletes.value = new Set([...pendingDeletes.value].filter((i) => i !== rowIndex))
	}

	// ── Row-level operations ───────────────────────────

	function addNewRow() {
		const emptyRow: Record<string, unknown> = {}
		for (const col of columns()) {
			emptyRow[col.key] = col.nullable ? null : ""
		}
		newRows.value = [...newRows.value, emptyRow]
	}

	function removeNewRow(index: number) {
		newRows.value = newRows.value.filter((_, i) => i !== index)
	}

	function markRowDeleted(rowIndex: number) {
		const currentRows = rows()
		const newRowStartIndex = currentRows.length
		if (rowIndex >= newRowStartIndex) {
			// It's a new row, just remove it
			removeNewRow(rowIndex - newRowStartIndex)
			return
		}
		pendingDeletes.value = new Set([...pendingDeletes.value, rowIndex])
	}

	function unmarkRowDeleted(rowIndex: number) {
		pendingDeletes.value = new Set([...pendingDeletes.value].filter((i) => i !== rowIndex))
	}

	// ── Row status ─────────────────────────────────────

	function getRowStatus(rowIndex: number): RowStatus {
		const currentRows = rows()
		if (rowIndex >= currentRows.length) return "new"
		if (pendingDeletes.value.has(rowIndex)) return "deleted"
		if (dirtyRows.value.has(rowIndex)) return "dirty"
		return "clean"
	}

	/** Get the effective (display) value for a cell, taking dirty changes into account. */
	function getCellValue(rowIndex: number, columnKey: string): unknown {
		const currentRows = rows()
		const newRowStartIndex = currentRows.length
		if (rowIndex >= newRowStartIndex) {
			const newRowIdx = rowIndex - newRowStartIndex
			return newRows.value[newRowIdx]?.[columnKey] ?? null
		}
		const rowDirty = dirtyRows.value.get(rowIndex)
		if (rowDirty?.has(columnKey)) {
			return rowDirty.get(columnKey)
		}
		return currentRows[rowIndex]?.[columnKey] ?? null
	}

	function isCellDirty(rowIndex: number, columnKey: string): boolean {
		return dirtyRows.value.get(rowIndex)?.has(columnKey) ?? false
	}

	// ── Changeset generation ───────────────────────────

	/**
	 * Build a filter expression to locate a specific row using all scalar column values.
	 * This is Strategy A from the design doc.
	 */
	function buildRowFilter(rowIndex: number): string {
		const currentRows = rows()
		const row = currentRows[rowIndex]
		if (!row) return "1 = 0"

		const cols = columns()
		const conditions: string[] = []
		for (const col of cols) {
			// Skip vector/binary columns
			if (col.editable === false) continue
			const value = row[col.key]
			if (value === null || value === undefined) {
				conditions.push(`${col.key} IS NULL`)
			} else if (typeof value === "number") {
				conditions.push(`${col.key} = ${value}`)
			} else if (typeof value === "boolean") {
				conditions.push(`${col.key} = ${value}`)
			} else {
				conditions.push(`${col.key} = '${String(value).replace(/'/g, "''")}'`)
			}
		}
		return conditions.length ? conditions.join(" AND ") : "1 = 0"
	}

	function generateChangeset(): DataGridChangeset {
		const changeset: DataGridChangeset = {
			updated: [],
			inserted: [],
			deleted: [],
		}

		// Deleted rows
		for (const rowIndex of pendingDeletes.value) {
			changeset.deleted.push(buildRowFilter(rowIndex))
		}

		// Updated rows (skip deleted ones)
		for (const [rowIndex, colMap] of dirtyRows.value) {
			if (pendingDeletes.value.has(rowIndex)) continue
			const updates: Array<{ column: string; expr: string }> = []
			for (const [columnKey, newValue] of colMap) {
				if (newValue === null || newValue === undefined) {
					updates.push({ column: columnKey, expr: "NULL" })
				} else if (typeof newValue === "number") {
					updates.push({ column: columnKey, expr: String(newValue) })
				} else if (typeof newValue === "boolean") {
					updates.push({ column: columnKey, expr: String(newValue) })
				} else {
					updates.push({
						column: columnKey,
						expr: `'${String(newValue).replace(/'/g, "''")}'`,
					})
				}
			}
			if (updates.length) {
				changeset.updated.push({
					filter: buildRowFilter(rowIndex),
					updates,
				})
			}
		}

		// Inserted rows
		changeset.inserted = [...newRows.value]

		return changeset
	}

	function clearAllChanges() {
		dirtyRows.value = new Map()
		newRows.value = []
		pendingDeletes.value = new Set()
		activeCell.value = null
	}

	// ── Navigation helpers ─────────────────────────────

	function moveToNextCell(direction: "right" | "left" | "down" | "up") {
		const ac = activeCell.value
		if (!ac) return
		const cols = columns().filter((c) => c.editable !== false)
		const allRows = rows()
		const totalRows = allRows.length + newRows.value.length
		const colIdx = cols.findIndex((c) => c.key === ac.columnKey)
		if (colIdx === -1) return

		let nextRow = ac.rowIndex
		let nextCol = colIdx

		switch (direction) {
			case "right":
				nextCol = colIdx + 1 < cols.length ? colIdx + 1 : colIdx
				break
			case "left":
				nextCol = colIdx > 0 ? colIdx - 1 : 0
				break
			case "down":
				nextRow = ac.rowIndex + 1 < totalRows ? ac.rowIndex + 1 : ac.rowIndex
				break
			case "up":
				nextRow = ac.rowIndex > 0 ? ac.rowIndex - 1 : 0
				break
		}

		activeCell.value = { rowIndex: nextRow, columnKey: cols[nextCol].key }
	}

	return {
		activeCell,
		dirtyRows,
		newRows,
		pendingDeletes,
		hasChanges,
		dirtyCount,
		startEditing,
		stopEditing,
		isEditing,
		commitCellEdit,
		cancelCellEdit,
		revertRow,
		addNewRow,
		removeNewRow,
		markRowDeleted,
		unmarkRowDeleted,
		getRowStatus,
		getCellValue,
		isCellDirty,
		generateChangeset,
		clearAllChanges,
		moveToNextCell,
	}
}

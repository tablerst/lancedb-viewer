import type { VNodeChild } from "vue"

/** Column definition for the DataGrid. */
export interface DataGridColumn {
	/** Column key (matches the field name in row data). */
	key: string
	/** Display label. */
	label: string
	/** Arrow/LanceDB data type string (e.g. "Utf8", "Int64", "FixedSizeList(Float32, 3)"). */
	dataType: string
	/** Whether the column allows nulls. */
	nullable: boolean
	/** Column width in pixels. Undefined means auto. */
	width?: number
	/** Pin column to left or right. */
	fixed?: "left" | "right"
	/** Whether this column is editable. Defaults to true for scalar types. */
	editable?: boolean
	/** Custom cell renderer. Falls back to built-in formatters. */
	render?: (value: unknown) => VNodeChild
}

/** Sort descriptor. */
export interface DataGridSort {
	column: string
	order: "asc" | "desc"
}

/** Parameters emitted when the grid requests data. */
export interface DataGridQueryParams {
	offset: number
	limit: number
	/** Per-column filter values (raw user input). */
	columnFilters?: Record<string, string>
	/** Global filter expression (SQL-like). */
	globalFilter?: string
	/** Column projection. */
	projection?: string[]
	/** Sort. */
	sort?: DataGridSort
}

/** Row-level update descriptor. */
export interface RowUpdate {
	/** Filter expression to locate the row. */
	filter: string
	/** Column-expression pairs to apply. */
	updates: Array<{ column: string; expr: string }>
}

/** Aggregated changeset for batch save. */
export interface DataGridChangeset {
	/** Modified existing rows. */
	updated: RowUpdate[]
	/** Newly inserted rows. */
	inserted: Record<string, unknown>[]
	/** Filter expressions for rows to delete. */
	deleted: string[]
}

/** Row status for visual styling. */
export type RowStatus = "clean" | "dirty" | "new" | "deleted"

/** Editing state for a single cell. */
export interface CellAddress {
	rowIndex: number
	columnKey: string
}

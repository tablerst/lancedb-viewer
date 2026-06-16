<script setup lang="ts">
/**
 * DataGrid — custom table component for LanceDB Studio.
 *
 * Provides:
 * - Column header sorting via context menu (DBeaver-style)
 * - Per-column filter via context menu
 * - Inline cell editing (double-click)
 * - Dirty tracking + batch save
 * - Toolbar (refresh, add/save/export, batch ops)
 * - Status bar (pagination, row count, dirty count)
 */

import { ArrowDown, ArrowUp, ChevronDown, Filter } from "lucide-vue-next"
import { computed, ref } from "vue"
import DataGridCell from "./DataGridCell.vue"
import DataGridHeaderMenu from "./DataGridHeaderMenu.vue"
import DataGridStatusBar from "./DataGridStatusBar.vue"
import DataGridToolbar from "./DataGridToolbar.vue"
import type { DataGridChangeset, DataGridColumn, DataGridQueryParams, DataGridSort } from "./types"
import { useDataGridEditing } from "./useDataGridEditing"
import { useDataGridFilters } from "./useDataGridFilters"

const props = withDefaults(
	defineProps<{
		columns: DataGridColumn[]
		rows: Record<string, unknown>[]
		loading?: boolean
		offset?: number
		limit?: number
		hasNextPage?: boolean
		totalRows?: number
		editable?: boolean
		error?: string
		showAdvancedFilter?: boolean
		globalFilter?: string
		loadTimeMs?: number | null
	}>(),
	{
		loading: false,
		offset: 0,
		limit: 50,
		hasNextPage: false,
		editable: true,
		error: "",
		showAdvancedFilter: false,
		globalFilter: "",
		loadTimeMs: null,
	}
)

const emit = defineEmits<{
	(e: "query", params: DataGridQueryParams): void
	(e: "save", changeset: DataGridChangeset): void
	(e: "refresh"): void
	(e: "export"): void
	(e: "toggle-advanced-filter"): void
	(e: "open-batch-write"): void
	(e: "open-batch-update"): void
	(e: "open-batch-delete"): void
}>()

// ── Filters ────────────────────────────────────────────

const { columnFilters, setFilter, clearFilter, activeFilterCount } = useDataGridFilters()

// ── Sorting ────────────────────────────────────────────

const sort = ref<DataGridSort | null>(null)

function setSort(columnKey: string, order: "asc" | "desc" | null) {
	if (order === null) {
		sort.value = null
	} else {
		sort.value = { column: columnKey, order }
	}
}

// ── Header context menu ────────────────────────────────

const headerMenu = ref<{
	column: DataGridColumn
	x: number
	y: number
} | null>(null)

function openHeaderMenu(col: DataGridColumn, event: MouseEvent) {
	event.preventDefault()
	event.stopPropagation()
	headerMenu.value = {
		column: col,
		x: Math.min(event.clientX, window.innerWidth - 260),
		y: Math.min(event.clientY, window.innerHeight - 400),
	}
}

function handleMenuSort(order: "asc" | "desc" | null) {
	if (!headerMenu.value) return
	setSort(headerMenu.value.column.key, order)
}

function handleMenuFilter(column: string, value: string) {
	setFilter(column, value)
	emitQuery(0) // reset page on filter change
}

function handleMenuClearFilter(column: string) {
	clearFilter(column)
	emitQuery(0)
}

function closeHeaderMenu() {
	headerMenu.value = null
}

// ── Editing ────────────────────────────────────────────

const editing = useDataGridEditing(
	() => props.columns,
	() => props.rows
)

// ── Derived data ───────────────────────────────────────

const sortedRows = computed(() => {
	const data = [...props.rows]
	if (!sort.value) return data
	const { column, order } = sort.value
	const dir = order === "asc" ? 1 : -1
	return data.sort((a, b) => {
		const va = a[column]
		const vb = b[column]
		if (va === vb) return 0
		if (va === null || va === undefined) return -1 * dir
		if (vb === null || vb === undefined) return 1 * dir
		const na = typeof va === "number" ? va : Number(va)
		const nb = typeof vb === "number" ? vb : Number(vb)
		if (!Number.isNaN(na) && !Number.isNaN(nb)) return (na - nb) * dir
		return String(va).localeCompare(String(vb)) * dir
	})
})

/** All rows including new rows. */
const allRows = computed(() => {
	const rows = sortedRows.value.map((r) => ({ ...r }))
	for (const newRow of editing.newRows.value) {
		rows.push({ ...newRow })
	}
	return rows
})

// ── Pagination ─────────────────────────────────────────

const page = computed(() => Math.max(1, Math.floor(props.offset / props.limit) + 1))
const pageCount = computed(() => Math.max(1, props.hasNextPage ? page.value + 1 : page.value))

function handlePageChange(newPage: number) {
	if (props.loading) return
	emitQuery((newPage - 1) * props.limit, props.limit)
}

function handlePageSizeChange(newSize: number) {
	if (props.loading) return
	emitQuery(0, newSize)
}

// ── Query emission ─────────────────────────────────────

function emitQuery(offset?: number, limit?: number) {
	emit("query", {
		offset: offset ?? props.offset,
		limit: limit ?? props.limit,
		columnFilters: { ...columnFilters.value },
		globalFilter: props.globalFilter,
		sort: sort.value ?? undefined,
	})
}

// ── Toolbar actions ────────────────────────────────────

function handleRefresh() {
	emit("refresh")
}

function handleSave() {
	const changeset = editing.generateChangeset()
	emit("save", changeset)
}

function handleDiscard() {
	editing.clearAllChanges()
}

function handleAddRow() {
	editing.addNewRow()
}

// ── Cell editing callbacks ─────────────────────────────

function handleCellCommit(rowIndex: number, columnKey: string, value: unknown) {
	editing.commitCellEdit(rowIndex, columnKey, value)
}

function handleCellCancel() {
	editing.cancelCellEdit()
}

function handleCellStartEdit(rowIndex: number, columnKey: string) {
	if (!props.editable) return
	editing.startEditing(rowIndex, columnKey)
}

function handleCellNavigate(direction: "right" | "left" | "down" | "up") {
	editing.moveToNextCell(direction)
}

// ── Keyboard shortcuts ─────────────────────────────────

function handleKeydown(event: KeyboardEvent) {
	if (event.key === "F5") {
		event.preventDefault()
		handleRefresh()
		return
	}
	if ((event.ctrlKey || event.metaKey) && event.key === "s") {
		event.preventDefault()
		if (editing.hasChanges.value) handleSave()
		return
	}
	if ((event.ctrlKey || event.metaKey) && event.key === "n") {
		event.preventDefault()
		handleAddRow()
		return
	}
	if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "F") {
		event.preventDefault()
		emit("toggle-advanced-filter")
		return
	}
}

// ── Column resize ──────────────────────────────────────

const columnWidths = ref<Record<string, number>>({})

function startResize(columnKey: string, event: MouseEvent) {
	event.preventDefault()
	const startX = event.clientX
	const startWidth = columnWidths.value[columnKey] ?? 150

	function onMouseMove(e: MouseEvent) {
		const diff = e.clientX - startX
		const newWidth = Math.max(50, startWidth + diff)
		columnWidths.value = { ...columnWidths.value, [columnKey]: newWidth }
	}

	function onMouseUp() {
		document.removeEventListener("mousemove", onMouseMove)
		document.removeEventListener("mouseup", onMouseUp)
	}

	document.addEventListener("mousemove", onMouseMove)
	document.addEventListener("mouseup", onMouseUp)
}

function getColumnWidth(col: DataGridColumn): string | undefined {
	const w = columnWidths.value[col.key] ?? col.width
	return w ? `${w}px` : undefined
}

// ── Row status CSS class ───────────────────────────────

function rowClass(rowIndex: number): string {
	const status = editing.getRowStatus(rowIndex)
	switch (status) {
		case "dirty":
			return "datagrid-row--dirty"
		case "new":
			return "datagrid-row--new"
		case "deleted":
			return "datagrid-row--deleted"
		default:
			return ""
	}
}

/** Check if a column has an active filter. */
function hasColumnFilter(columnKey: string): boolean {
	return Boolean(columnFilters.value[columnKey]?.trim())
}
</script>

<template>
	<div
		class="datagrid-container"
		tabindex="0"
		@keydown="handleKeydown"
	>
		<!-- Error -->
		<NAlert v-if="error" type="error" :bordered="false" class="mb-2">
			{{ error }}
		</NAlert>

		<!-- Toolbar -->
		<DataGridToolbar
			:loading="loading"
			:disabled="!columns.length"
			:show-advanced-filter="showAdvancedFilter"
			:has-changes="editing.hasChanges.value"
			:dirty-count="editing.dirtyCount.value"
			:active-filter-count="activeFilterCount()"
			@refresh="handleRefresh"
			@toggle-advanced-filter="emit('toggle-advanced-filter')"
			@add-row="handleAddRow"
			@save="handleSave"
			@discard="handleDiscard"
			@export="emit('export')"
			@open-batch-write="emit('open-batch-write')"
			@open-batch-update="emit('open-batch-update')"
			@open-batch-delete="emit('open-batch-delete')"
		/>

		<!-- Advanced filter slot -->
		<slot name="advanced-filter" />

		<!-- Table -->
		<div class="datagrid-table-wrapper">
			<!-- Loading skeleton -->
			<div v-if="loading && !allRows.length" class="datagrid-skeleton">
				<div v-for="i in 8" :key="i" class="datagrid-skeleton-row" />
			</div>

			<table v-else class="datagrid-table">
				<thead>
					<!-- Header row -->
					<tr class="datagrid-header-row">
						<th
							v-for="col in columns"
							:key="col.key"
							class="datagrid-header-cell"
							:class="{ 'datagrid-header-cell--filtered': hasColumnFilter(col.key) }"
							:style="{ width: getColumnWidth(col), minWidth: getColumnWidth(col) }"
							@contextmenu="openHeaderMenu(col, $event)"
						>
							<div class="datagrid-header-content">
								<span class="datagrid-header-label" :title="col.label">
									{{ col.label }}
								</span>
								<span class="datagrid-header-indicators">
									<Filter
										v-if="hasColumnFilter(col.key)"
										class="h-3 w-3 text-sky-500"
									/>
									<template v-if="sort?.column === col.key">
										<ArrowUp v-if="sort.order === 'asc'" class="h-3 w-3 text-sky-500" />
										<ArrowDown v-else class="h-3 w-3 text-sky-500" />
									</template>
								</span>
								<button
									class="datagrid-header-menu-trigger"
									title="列菜单"
									:aria-label="`打开 ${col.label} 列菜单`"
									@click="openHeaderMenu(col, $event)"
									@mousedown.stop
								>
									<ChevronDown class="h-3 w-3" />
								</button>
							</div>
							<div
								class="datagrid-resize-handle"
								@mousedown.stop="startResize(col.key, $event)"
								@click.stop
							/>
						</th>
					</tr>
				</thead>
				<tbody>
					<tr
						v-for="(_row, rowIndex) in allRows"
						:key="rowIndex"
						class="datagrid-body-row"
						:class="rowClass(rowIndex)"
					>
						<DataGridCell
							v-for="col in columns"
							:key="col.key"
							:value="editing.getCellValue(rowIndex, col.key)"
							:column-key="col.key"
							:data-type="col.dataType"
							:nullable="col.nullable"
							:editable="editable && col.editable !== false"
							:editing="editing.isEditing(rowIndex, col.key)"
							:dirty="editing.isCellDirty(rowIndex, col.key)"
							:style="{ width: getColumnWidth(col), minWidth: getColumnWidth(col) }"
							@commit="(v: unknown) => handleCellCommit(rowIndex, col.key, v)"
							@cancel="handleCellCancel"
							@start-edit="handleCellStartEdit(rowIndex, col.key)"
							@navigate="handleCellNavigate"
						/>
					</tr>
					<tr v-if="!allRows.length && !loading">
						<td :colspan="columns.length" class="datagrid-empty">
							没有数据
						</td>
					</tr>
				</tbody>
			</table>

			<!-- Loading overlay -->
			<div v-if="loading && allRows.length" class="datagrid-loading-overlay">
				<div class="datagrid-loading-spinner" />
			</div>
		</div>

		<!-- Status bar -->
		<DataGridStatusBar
			:page="page"
			:page-size="limit"
			:page-count="pageCount"
			:row-count="allRows.length"
			:dirty-count="editing.dirtyCount.value"
			:loading="loading"
			:disabled="!columns.length"
			:load-time-ms="loadTimeMs"
			:has-next-page="hasNextPage"
			@update:page="handlePageChange"
			@update:page-size="handlePageSizeChange"
		/>

		<!-- Header context menu -->
		<DataGridHeaderMenu
			v-if="headerMenu"
			:column="headerMenu.column"
			:x="headerMenu.x"
			:y="headerMenu.y"
			:sort="sort"
			:filter-value="columnFilters[headerMenu.column.key] ?? ''"
			@sort="handleMenuSort"
			@filter="handleMenuFilter"
			@clear-filter="handleMenuClearFilter"
			@close="closeHeaderMenu"
		/>
	</div>
</template>

<style scoped>
.datagrid-container {
	border: 1px solid #e2e8f0;
	border-radius: 8px;
	overflow: hidden;
	font-size: 13px;
	display: flex;
	flex-direction: column;
	background: white;
	outline: none;
}

.datagrid-table-wrapper {
	overflow: auto;
	flex: 1;
	min-height: 0;
	position: relative;
}

.datagrid-table {
	width: 100%;
	border-collapse: collapse;
	table-layout: fixed;
}

/* Header */
.datagrid-header-row {
	background: #f8fafc;
}

.datagrid-header-cell {
	padding: 8px 12px;
	border-bottom: 1px solid #e2e8f0;
	font-weight: 600;
	color: #334155;
	text-align: left;
	white-space: nowrap;
	user-select: none;
	cursor: default;
	position: relative;
	overflow: hidden;
	text-overflow: ellipsis;
}

.datagrid-header-cell:hover {
	background: #f1f5f9;
}

.datagrid-header-cell--filtered {
	background: #f0f9ff;
}

.datagrid-header-content {
	display: flex;
	align-items: center;
	gap: 4px;
}

.datagrid-header-label {
	overflow: hidden;
	text-overflow: ellipsis;
	flex: 1;
}

.datagrid-header-indicators {
	display: flex;
	align-items: center;
	gap: 2px;
	flex-shrink: 0;
}

.datagrid-header-menu-trigger {
	display: none;
	align-items: center;
	justify-content: center;
	width: 18px;
	height: 18px;
	border: none;
	border-radius: 3px;
	background: transparent;
	color: #94a3b8;
	cursor: pointer;
	padding: 0;
	flex-shrink: 0;
	margin-left: 2px;
}

.datagrid-header-cell:hover .datagrid-header-menu-trigger {
	display: inline-flex;
}

.datagrid-header-menu-trigger:hover {
	background: #e2e8f0;
	color: #475569;
}

/* Resize handle */
.datagrid-resize-handle {
	position: absolute;
	right: 0;
	top: 0;
	bottom: 0;
	width: 4px;
	cursor: col-resize;
	background: transparent;
}

.datagrid-resize-handle:hover {
	background: #38bdf8;
}

/* Body rows */
.datagrid-body-row {
	border-bottom: 1px solid #f1f5f9;
	transition: background 0.1s;
}

.datagrid-body-row:hover {
	background: #f8fafc;
}

.datagrid-row--dirty {
	background: #fffbeb !important;
}

.datagrid-row--new {
	background: #f0fdf4 !important;
}

.datagrid-row--deleted {
	background: #fef2f2 !important;
	text-decoration: line-through;
	opacity: 0.6;
}

/* Empty state */
.datagrid-empty {
	padding: 40px;
	text-align: center;
	color: #94a3b8;
	font-style: italic;
}

/* Loading skeleton */
.datagrid-skeleton {
	padding: 12px;
}

.datagrid-skeleton-row {
	height: 28px;
	margin-bottom: 6px;
	background: linear-gradient(90deg, #f1f5f9 25%, #e2e8f0 50%, #f1f5f9 75%);
	background-size: 200% 100%;
	animation: skeleton-shimmer 1.5s ease-in-out infinite;
	border-radius: 4px;
}

@keyframes skeleton-shimmer {
	0% { background-position: 200% 0; }
	100% { background-position: -200% 0; }
}

/* Loading overlay */
.datagrid-loading-overlay {
	position: absolute;
	inset: 0;
	background: rgba(255, 255, 255, 0.6);
	display: flex;
	align-items: center;
	justify-content: center;
	z-index: 5;
}

.datagrid-loading-spinner {
	width: 24px;
	height: 24px;
	border: 3px solid #e2e8f0;
	border-top-color: #38bdf8;
	border-radius: 50%;
	animation: spin 0.8s linear infinite;
}

@keyframes spin {
	to { transform: rotate(360deg); }
}
</style>

import { ref } from "vue"

/**
 * Manages per-column filter state for the DataGrid filter row.
 */
export function useDataGridFilters() {
	const columnFilters = ref<Record<string, string>>({})
	const showFilterRow = ref(true)

	function setFilter(column: string, value: string) {
		columnFilters.value = { ...columnFilters.value, [column]: value }
	}

	function clearFilter(column: string) {
		const next = { ...columnFilters.value }
		delete next[column]
		columnFilters.value = next
	}

	function clearAllFilters() {
		columnFilters.value = {}
	}

	function toggleFilterRow() {
		showFilterRow.value = !showFilterRow.value
	}

	/** Number of active (non-empty) filters. */
	function activeFilterCount(): number {
		return Object.values(columnFilters.value).filter((v) => v.trim()).length
	}

	return {
		columnFilters,
		showFilterRow,
		setFilter,
		clearFilter,
		clearAllFilters,
		toggleFilterRow,
		activeFilterCount,
	}
}

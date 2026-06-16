import type { DeleteRowsRequestV1, UpdateColumnInputV1, UpdateRowsRequestV1 } from "../../ipc/v1"

export function normalizeMutationFilter(filter: string | undefined): string | undefined {
	const cleaned = filter?.trim()
	return cleaned ? cleaned : undefined
}

export function isTriviallyBroadMutationFilter(filter: string): boolean {
	const normalized = filter.replace(/\s+/g, "").toLowerCase()
	return normalized === "true" || normalized === "1=1"
}

export function buildUpdateRowsMutationRequest(
	tableId: string,
	filter: string | undefined,
	updates: UpdateColumnInputV1[]
): UpdateRowsRequestV1 {
	const cleanedFilter = normalizeMutationFilter(filter)
	const allowFullTable = !cleanedFilter || isTriviallyBroadMutationFilter(cleanedFilter)

	return {
		tableId,
		filter: cleanedFilter,
		updates,
		allowFullTable,
	}
}

export function buildDeleteRowsMutationRequest(
	tableId: string,
	filter: string
): DeleteRowsRequestV1 | null {
	const cleanedFilter = normalizeMutationFilter(filter)
	if (!cleanedFilter) {
		return null
	}

	return {
		tableId,
		filter: cleanedFilter,
		allowFullTable: isTriviallyBroadMutationFilter(cleanedFilter),
	}
}

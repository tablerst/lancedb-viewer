import { describe, expect, it } from "vitest"

import {
	buildDeleteRowsMutationRequest,
	buildUpdateRowsMutationRequest,
	isTriviallyBroadMutationFilter,
	normalizeMutationFilter,
} from "./mutationGuards"

describe("mutation guard request builders", () => {
	it("normalizes blank filters", () => {
		expect(normalizeMutationFilter(undefined)).toBeUndefined()
		expect(normalizeMutationFilter("   ")).toBeUndefined()
		expect(normalizeMutationFilter(" id = 1 ")).toBe("id = 1")
	})

	it("detects trivially broad mutation filters", () => {
		expect(isTriviallyBroadMutationFilter("true")).toBe(true)
		expect(isTriviallyBroadMutationFilter(" TRUE ")).toBe(true)
		expect(isTriviallyBroadMutationFilter("1 = 1")).toBe(true)
		expect(isTriviallyBroadMutationFilter("id = 1")).toBe(false)
	})

	it("opts full-table updates in only for empty or trivially broad filters", () => {
		expect(
			buildUpdateRowsMutationRequest("table-1", undefined, [{ column: "text", expr: "'updated'" }])
		).toEqual({
			tableId: "table-1",
			filter: undefined,
			updates: [{ column: "text", expr: "'updated'" }],
			allowFullTable: true,
		})

		expect(
			buildUpdateRowsMutationRequest("table-1", " 1 = 1 ", [{ column: "text", expr: "'updated'" }])
		).toMatchObject({
			filter: "1 = 1",
			allowFullTable: true,
		})

		expect(
			buildUpdateRowsMutationRequest("table-1", "id = 1", [{ column: "text", expr: "'updated'" }])
		).toMatchObject({
			filter: "id = 1",
			allowFullTable: false,
		})
	})

	it("opts full-table deletes in only for explicit broad filters", () => {
		expect(buildDeleteRowsMutationRequest("table-1", " ")).toBeNull()
		expect(buildDeleteRowsMutationRequest("table-1", " true ")).toEqual({
			tableId: "table-1",
			filter: "true",
			allowFullTable: true,
		})
		expect(buildDeleteRowsMutationRequest("table-1", "id = 1")).toEqual({
			tableId: "table-1",
			filter: "id = 1",
			allowFullTable: false,
		})
	})
})

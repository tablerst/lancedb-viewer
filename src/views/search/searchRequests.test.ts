import { describe, expect, it } from "vitest"
import {
	buildCombinedSearchRequest,
	buildFilterQueryRequest,
	buildFtsSearchRequest,
	buildVectorSearchRequest,
	parseVectorInputValue,
} from "./searchRequests"

describe("search request builders", () => {
	it("rejects blank filter queries before IPC", () => {
		const result = buildFilterQueryRequest({
			tableId: "tbl",
			filter: "  ",
			projection: [],
			limit: 50,
			offset: 0,
		})

		expect(result).toEqual({ ok: false, message: "请输入过滤表达式" })
	})

	it("trims valid filter queries and omits empty projection", () => {
		const result = buildFilterQueryRequest({
			tableId: "tbl",
			filter: " id > 10 ",
			projection: [],
			limit: 25,
			offset: 5,
		})

		expect(result).toEqual({
			ok: true,
			request: {
				tableId: "tbl",
				filter: "id > 10",
				projection: undefined,
				limit: 25,
				offset: 5,
			},
		})
	})

	it("parses comma, whitespace, and bracket vector input", () => {
		expect(parseVectorInputValue("[0.1, 0.2 0.3]")).toEqual([0.1, 0.2, 0.3])
		expect(parseVectorInputValue("")).toBeNull()
		expect(parseVectorInputValue("0.1, nope")).toBeNull()
	})

	it("builds vector search requests with optional fields normalized", () => {
		const result = buildVectorSearchRequest({
			tableId: "tbl",
			vectorText: "1, 2, 3",
			column: null,
			topK: 10,
			offset: 2,
			projection: ["id"],
			filter: " ",
			nprobes: null,
			refineFactor: 4,
		})

		expect(result).toEqual({
			ok: true,
			request: {
				tableId: "tbl",
				vector: [1, 2, 3],
				column: undefined,
				topK: 10,
				offset: 2,
				projection: ["id"],
				filter: undefined,
				nprobes: undefined,
				refineFactor: 4,
			},
		})
	})

	it("rejects blank full-text queries before IPC", () => {
		const result = buildFtsSearchRequest({
			tableId: "tbl",
			query: "\t",
			columns: [],
			limit: 50,
			offset: 0,
			projection: [],
			filter: "",
		})

		expect(result).toEqual({ ok: false, message: "请输入查询文本" })
	})

	it("accepts combined search when either query text or vector input exists", () => {
		expect(
			buildCombinedSearchRequest({
				tableId: "tbl",
				query: "hello",
				vectorText: "",
				vectorColumn: null,
				columns: ["text"],
				limit: 20,
				offset: 0,
				projection: [],
				filter: " category = 'note' ",
				nprobes: null,
				refineFactor: null,
			})
		).toEqual({
			ok: true,
			request: {
				tableId: "tbl",
				vector: undefined,
				vectorColumn: undefined,
				query: "hello",
				columns: ["text"],
				projection: undefined,
				filter: "category = 'note'",
				limit: 20,
				offset: 0,
				nprobes: undefined,
				refineFactor: undefined,
			},
		})

		expect(
			buildCombinedSearchRequest({
				tableId: "tbl",
				query: "",
				vectorText: "0.1, 0.2",
				vectorColumn: "vector",
				columns: [],
				limit: 20,
				offset: 0,
				projection: ["id"],
				filter: "",
				nprobes: 8,
				refineFactor: null,
			})
		).toEqual({
			ok: true,
			request: {
				tableId: "tbl",
				vector: [0.1, 0.2],
				vectorColumn: "vector",
				query: undefined,
				columns: undefined,
				projection: ["id"],
				filter: undefined,
				limit: 20,
				offset: 0,
				nprobes: 8,
				refineFactor: undefined,
			},
		})
	})

	it("rejects combined search with no inputs or invalid vector input", () => {
		expect(
			buildCombinedSearchRequest({
				tableId: "tbl",
				query: "",
				vectorText: "",
				vectorColumn: null,
				columns: [],
				limit: 20,
				offset: 0,
				projection: [],
				filter: "",
				nprobes: null,
				refineFactor: null,
			})
		).toEqual({ ok: false, message: "请输入向量或查询文本" })

		expect(
			buildCombinedSearchRequest({
				tableId: "tbl",
				query: "",
				vectorText: "oops",
				vectorColumn: null,
				columns: [],
				limit: 20,
				offset: 0,
				projection: [],
				filter: "",
				nprobes: null,
				refineFactor: null,
			})
		).toEqual({ ok: false, message: "请输入有效向量（例如：0.1, 0.2, 0.3）" })
	})
})

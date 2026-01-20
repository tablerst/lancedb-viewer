import { describe, expect, it } from "vitest"

import { formatCellValue, normalizeRow } from "./formatters"

describe("formatCellValue", () => {
	it("formats primitives", () => {
		expect(formatCellValue(null)).toBe("")
		expect(formatCellValue(undefined)).toBe("")
		expect(formatCellValue(42)).toBe("42")
		expect(formatCellValue(true)).toBe("true")
		expect(formatCellValue("hello")).toBe("hello")
	})

	it("formats arrays with preview", () => {
		expect(formatCellValue([1, 2, 3])).toBe("[1, 2, 3]")
		expect(formatCellValue([1, 2, 3, 4, 5, 6, 7])).toBe("[1, 2, 3, 4, 5, 6 …]")
	})

	it("formats objects as json", () => {
		expect(formatCellValue({ id: 1, name: "Alice" })).toBe('{"id":1,"name":"Alice"}')
	})
})

describe("normalizeRow", () => {
	it("keeps object rows", () => {
		expect(normalizeRow({ id: 1 })).toEqual({ id: 1 })
	})

	it("wraps non-object rows", () => {
		expect(normalizeRow("value")).toEqual({ value: "value" })
	})
})

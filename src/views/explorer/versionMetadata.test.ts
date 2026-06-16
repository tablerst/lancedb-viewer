import { describe, expect, it } from "vitest"
import { formatMetadata, getMetadataEntries } from "./versionMetadata"

describe("version metadata helpers", () => {
	it("returns stable key/value entries for structured version metadata display", () => {
		expect(
			getMetadataEntries({
				operation: "write",
				user: "studio",
			})
		).toEqual([
			{ key: "operation", value: "write" },
			{ key: "user", value: "studio" },
		])
	})

	it("keeps the compact metadata string formatter for legacy contexts", () => {
		expect(formatMetadata({ operation: "write", user: "studio" })).toBe(
			"operation=write, user=studio"
		)
		expect(formatMetadata({})).toBe("—")
	})
})

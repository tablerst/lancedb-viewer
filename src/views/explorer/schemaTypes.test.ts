import { describe, expect, it } from "vitest"
import { formatSchemaDataType } from "./schemaTypes"

describe("schema type formatting", () => {
	it("summarizes LanceDB vector fields without leaking Arrow debug internals", () => {
		expect(
			formatSchemaDataType(
				'FixedSizeList(Field { name: "item", data_type: Float32, nullable: true, dict_id: 0, dict_is_ordered: false, metadata: {} }, 1024)'
			)
		).toBe("Vector<Float32, 1024>")
	})

	it("summarizes non-item fixed-size lists with field name and dimension", () => {
		expect(
			formatSchemaDataType(
				'FixedSizeList(Field { name: "values", data_type: Int64, nullable: true, dict_id: 0, dict_is_ordered: false, metadata: {} }, 3)'
			)
		).toBe("FixedSizeList<values: Int64, 3>")
	})

	it("summarizes struct field names and types", () => {
		expect(
			formatSchemaDataType(
				'Struct([Field { name: "chapter_id", data_type: Int64, nullable: true, dict_id: 0, dict_is_ordered: false, metadata: {} }, Field { name: "chapter_title", data_type: Utf8, nullable: true, dict_id: 0, dict_is_ordered: false, metadata: {} }])'
			)
		).toBe("Struct<chapter_id: Int64, chapter_title: Utf8>")
	})

	it("keeps simple data types unchanged", () => {
		expect(formatSchemaDataType("Utf8")).toBe("Utf8")
	})

	it("keeps unsupported struct strings unchanged", () => {
		expect(formatSchemaDataType("Struct([])")).toBe("Struct([])")
	})
})

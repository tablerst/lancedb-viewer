const FIXED_SIZE_LIST_PATTERN =
	/^FixedSizeList\(Field \{ name: "([^"]+)", data_type: ([^,]+), .* \}, (\d+)\)$/
const STRUCT_FIELD_PATTERN = /Field \{ name: "([^"]+)", data_type: ([^,}\]]+)/g

export function formatSchemaDataType(dataType: string): string {
	const fixedSizeList = dataType.match(FIXED_SIZE_LIST_PATTERN)
	if (fixedSizeList) {
		const [, fieldName, itemType, length] = fixedSizeList
		const normalizedItem = itemType.trim()
		if (fieldName === "item") {
			return `Vector<${normalizedItem}, ${length}>`
		}
		return `FixedSizeList<${fieldName}: ${normalizedItem}, ${length}>`
	}

	if (dataType.startsWith("Struct([")) {
		const fields = [...dataType.matchAll(STRUCT_FIELD_PATTERN)].map(
			(match) => `${match[1]}: ${match[2].trim()}`
		)
		if (fields.length) {
			return `Struct<${fields.join(", ")}>`
		}
	}

	return dataType
}

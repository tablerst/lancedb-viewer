import { tableFromIPC } from "apache-arrow"

import type { DataChunk, SchemaDefinition } from "../ipc/v1"

type ArrowChunk = Extract<DataChunk, { format: "arrow" }>

export interface DecodedArrowChunk {
	rows: Record<string, unknown>[]
	schema: SchemaDefinition
}

function base64ToBytes(value: string): Uint8Array {
	const binary = globalThis.atob(value)
	const bytes = new Uint8Array(binary.length)
	for (let index = 0; index < binary.length; index += 1) {
		bytes[index] = binary.charCodeAt(index)
	}
	return bytes
}

function mapToRecord(value: Map<unknown, unknown>): Record<string, unknown> {
	const record: Record<string, unknown> = {}
	for (const [key, item] of value.entries()) {
		record[String(key)] = normalizeArrowValue(item)
	}
	return record
}

function arrayBufferViewToArray(value: ArrayBufferView): unknown[] {
	if (value instanceof DataView) {
		return Array.from(new Uint8Array(value.buffer, value.byteOffset, value.byteLength))
	}
	const typedArray = value as
		| Int8Array
		| Uint8Array
		| Uint8ClampedArray
		| Int16Array
		| Uint16Array
		| Int32Array
		| Uint32Array
		| Float32Array
		| Float64Array
		| BigInt64Array
		| BigUint64Array
	return Array.from(typedArray, normalizeArrowValue)
}

function normalizeArrowValue(value: unknown): unknown {
	if (value === null || value === undefined) {
		return value
	}
	if (typeof value === "bigint") {
		return value.toString()
	}
	if (value instanceof Date) {
		return value.toISOString()
	}
	if (value instanceof Map) {
		return mapToRecord(value)
	}
	if (ArrayBuffer.isView(value)) {
		return arrayBufferViewToArray(value)
	}
	if (Array.isArray(value)) {
		return value.map(normalizeArrowValue)
	}
	if (typeof value === "object") {
		const maybeVector = value as { toArray?: () => unknown }
		if (typeof maybeVector.toArray === "function") {
			const arrayValue = maybeVector.toArray()
			if (ArrayBuffer.isView(arrayValue) || Array.isArray(arrayValue)) {
				return Array.from(arrayValue as ArrayLike<unknown>, normalizeArrowValue)
			}
		}

		return Object.fromEntries(
			Object.entries(value as Record<string, unknown>).map(([key, item]) => [
				key,
				normalizeArrowValue(item),
			])
		)
	}
	return value
}

export function decodeArrowChunk(chunk: ArrowChunk): DecodedArrowChunk {
	const table = tableFromIPC(base64ToBytes(chunk.ipcBase64))
	const schema: SchemaDefinition = {
		fields: table.schema.fields.map((field) => ({
			name: field.name,
			dataType: String(field.type),
			nullable: field.nullable,
			metadata: field.metadata.size ? Object.fromEntries(field.metadata.entries()) : undefined,
		})),
	}
	const rows: Record<string, unknown>[] = []

	for (let rowIndex = 0; rowIndex < table.numRows; rowIndex += 1) {
		const row: Record<string, unknown> = {}
		for (const field of table.schema.fields) {
			row[field.name] = normalizeArrowValue(table.getChild(field.name)?.get(rowIndex))
		}
		rows.push(row)
	}

	return { rows, schema }
}

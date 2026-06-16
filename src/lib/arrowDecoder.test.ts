import { tableFromArrays, tableToIPC } from "apache-arrow"
import { describe, expect, it } from "vitest"

import { decodeArrowChunk } from "./arrowDecoder"

function toBase64(bytes: Uint8Array) {
	let binary = ""
	for (const byte of bytes) {
		binary += String.fromCharCode(byte)
	}
	return btoa(binary)
}

describe("decodeArrowChunk", () => {
	it("decodes Arrow IPC into DataGrid rows and schema", () => {
		const table = tableFromArrays({
			id: new Int32Array([1, 2]),
			name: ["alpha", "beta"],
			big: [1n, 2n],
			vector: [new Float32Array([1, 2]), new Float32Array([3, 4])],
		})

		const decoded = decodeArrowChunk({
			format: "arrow",
			ipcBase64: toBase64(tableToIPC(table, "stream")),
		})

		expect(decoded.schema.fields.map((field) => field.name)).toEqual([
			"id",
			"name",
			"big",
			"vector",
		])
		expect(decoded.rows).toEqual([
			{ id: 1, name: "alpha", big: "1", vector: [1, 2] },
			{ id: 2, name: "beta", big: "2", vector: [3, 4] },
		])
	})

	it("surfaces invalid IPC payloads as decode errors", () => {
		expect(() =>
			decodeArrowChunk({
				format: "arrow",
				ipcBase64: "not valid base64",
			})
		).toThrow()
	})
})

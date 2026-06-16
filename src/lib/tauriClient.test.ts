import { beforeEach, describe, expect, it, vi } from "vitest"

import { connectV1, createIndexV1, deleteRowsV1, updateRowsV1 } from "./tauriClient"

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock("@tauri-apps/api/core", () => ({
	invoke: invokeMock,
}))

describe("tauriClient write mutation commands", () => {
	beforeEach(() => {
		invokeMock.mockReset()
		invokeMock.mockResolvedValue({ apiVersion: "v1", ok: true, data: {} })
	})

	it("passes explicit full-table update opt-in to the IPC command", async () => {
		await updateRowsV1({
			tableId: "table-1",
			updates: [{ column: "text", expr: "'updated'" }],
			allowFullTable: true,
		})

		expect(invokeMock).toHaveBeenCalledWith("update_rows_v1", {
			request: {
				tableId: "table-1",
				updates: [{ column: "text", expr: "'updated'" }],
				allowFullTable: true,
			},
		})
	})

	it("passes explicit full-table delete opt-in to the IPC command", async () => {
		await deleteRowsV1({
			tableId: "table-1",
			filter: "true",
			allowFullTable: true,
		})

		expect(invokeMock).toHaveBeenCalledWith("delete_rows_v1", {
			request: {
				tableId: "table-1",
				filter: "true",
				allowFullTable: true,
			},
		})
	})

	it("passes index creation tuning parameters to the IPC command", async () => {
		await createIndexV1({
			tableId: "table-1",
			columns: ["vector"],
			indexType: "ivf_hnsw_pq",
			name: "vector_hybrid",
			replace: true,
			distanceType: "cosine",
			numPartitions: 16,
			sampleRate: 128,
			maxIterations: 20,
			targetPartitionSize: 4096,
			numSubVectors: 4,
			numBits: 8,
			numEdges: 32,
			efConstruction: 256,
		})

		expect(invokeMock).toHaveBeenCalledWith("create_index_v1", {
			request: {
				tableId: "table-1",
				columns: ["vector"],
				indexType: "ivf_hnsw_pq",
				name: "vector_hybrid",
				replace: true,
				distanceType: "cosine",
				numPartitions: 16,
				sampleRate: 128,
				maxIterations: 20,
				targetPartitionSize: 4096,
				numSubVectors: 4,
				numBits: 8,
				numEdges: 32,
				efConstruction: 256,
			},
		})
	})

	it("returns an unsupported envelope for secret_ref auth without invoking IPC", async () => {
		const response = await connectV1({
			name: "secret",
			uri: "s3://bucket/path",
			auth: { type: "secret_ref", provider: "s3", reference: "stored-key" },
		})

		expect(response).toEqual({
			apiVersion: "v1",
			ok: false,
			error: {
				code: "not_implemented",
				message: "secret_ref auth is not supported in this version; use no auth or inline auth",
			},
		})
		expect(invokeMock).not.toHaveBeenCalled()
	})
})

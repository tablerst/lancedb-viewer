import { beforeEach, describe, expect, it, vi } from "vitest"

import { deleteRowsV1, updateRowsV1 } from "./tauriClient"

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
})

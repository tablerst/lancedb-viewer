import type { Ref } from "vue"
import { computed, ref, watch } from "vue"

import type { SchemaDefinition, TableInfo } from "../ipc/v1"
import {
	connectV1,
	getSchemaV1,
	listTablesV1,
	openTableV1,
	unwrapEnvelope,
} from "../lib/tauriClient"
import type { StoredProfile } from "../models/profile"
import { toConnectProfile } from "../models/profile"

export interface ConnectionState {
	connectionId: Ref<string | null>
	tables: Ref<TableInfo[]>
	activeTableName: Ref<string | null>
	activeTableId: Ref<string | null>
	schema: Ref<SchemaDefinition | null>
	isConnecting: Ref<boolean>
	isRefreshing: Ref<boolean>
	isOpening: Ref<boolean>
}

interface UseConnectionOptions {
	onStatus?: (message: string) => void
	onError?: (message: string) => void
}

function createConnectionState(): ConnectionState {
	return {
		connectionId: ref<string | null>(null),
		tables: ref<TableInfo[]>([]),
		activeTableName: ref<string | null>(null),
		activeTableId: ref<string | null>(null),
		schema: ref<SchemaDefinition | null>(null),
		isConnecting: ref(false),
		isRefreshing: ref(false),
		isOpening: ref(false),
	}
}

export function useConnection(
	profiles: Ref<StoredProfile[]>,
	activeProfileId: Ref<string | null>,
	options: UseConnectionOptions = {}
) {
	const connectionStates = ref<Record<string, ConnectionState>>({})

	function getState(profileId: string) {
		const existing = connectionStates.value[profileId]
		if (existing) {
			return existing
		}
		const next = createConnectionState()
		connectionStates.value = { ...connectionStates.value, [profileId]: next }
		return next
	}

	watch(
		() => profiles.value,
		(next) => {
			next.forEach((profile) => {
				getState(profile.id)
			})
		},
		{ immediate: true }
	)

	const activeConnection = computed(() => {
		const id = activeProfileId.value
		return id ? getState(id) : null
	})

	const connectionId = computed(() => activeConnection.value?.connectionId.value ?? null)
	const tables = computed(() => activeConnection.value?.tables.value ?? [])
	const activeTableName = computed(() => activeConnection.value?.activeTableName.value ?? null)
	const activeTableId = computed(() => activeConnection.value?.activeTableId.value ?? null)
	const schema = computed(() => activeConnection.value?.schema.value ?? null)
	const isConnecting = computed(() => activeConnection.value?.isConnecting.value ?? false)
	const isRefreshing = computed(() => activeConnection.value?.isRefreshing.value ?? false)
	const isOpening = computed(() => activeConnection.value?.isOpening.value ?? false)

	function resetConnection(profileId: string) {
		const state = getState(profileId)
		state.connectionId.value = null
		state.tables.value = []
		state.activeTableName.value = null
		state.activeTableId.value = null
		state.schema.value = null
	}

	async function connectProfile(profileId: string) {
		const profile = profiles.value.find((item) => item.id === profileId) ?? null
		if (!profile) {
			options.onError?.("连接档案不存在")
			return
		}
		const state = getState(profileId)
		if (state.isConnecting.value) {
			return
		}

		try {
			state.isConnecting.value = true
			resetConnection(profileId)
			const connectProfile = toConnectProfile(profile)
			connectProfile.auth ??= { type: "none" }
			const response = unwrapEnvelope(await connectV1(connectProfile))
			state.connectionId.value = response.connectionId
			options.onStatus?.(`已连接：${response.name}`)
			await refreshTables(profileId)
		} catch (error) {
			const message = error instanceof Error ? error.message : "连接失败"
			options.onError?.(message)
		} finally {
			state.isConnecting.value = false
		}
	}

	async function refreshTables(profileId: string) {
		const state = getState(profileId)
		const id = state.connectionId.value
		if (!id) {
			return
		}
		if (state.isRefreshing.value) {
			return
		}

		try {
			state.isRefreshing.value = true
			const response = unwrapEnvelope(await listTablesV1(id))
			state.tables.value = response.tables
		} catch (error) {
			const message = error instanceof Error ? error.message : "拉取表列表失败"
			options.onError?.(message)
		} finally {
			state.isRefreshing.value = false
		}
	}

	async function openTable(profileId: string, name: string) {
		const state = getState(profileId)
		const id = state.connectionId.value
		if (!id) {
			return
		}
		if (state.isOpening.value) {
			return
		}

		try {
			state.isOpening.value = true
			state.activeTableName.value = name
			state.activeTableId.value = null
			state.schema.value = null
			const handle = unwrapEnvelope(await openTableV1(id, name))
			state.activeTableId.value = handle.tableId
			state.schema.value = unwrapEnvelope(await getSchemaV1(handle.tableId))
		} catch (error) {
			const message = error instanceof Error ? error.message : "打开表失败"
			options.onError?.(message)
		} finally {
			state.isOpening.value = false
		}
	}

	return {
		connectionStates,
		activeConnection,
		connectionId,
		tables,
		activeTableName,
		activeTableId,
		schema,
		isConnecting,
		isRefreshing,
		isOpening,
		connectProfile,
		refreshTables,
		openTable,
		resetConnection,
	}
}

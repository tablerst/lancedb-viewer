import type { ComputedRef } from "vue";
import { ref } from "vue";

import type { SchemaDefinition, TableInfo } from "../ipc/v1";
import type { StoredProfile } from "../models/profile";
import { toConnectProfile } from "../models/profile";
import { connectV1, getSchemaV1, listTablesV1, openTableV1, unwrapEnvelope } from "../lib/tauriClient";

interface UseConnectionOptions {
	onStatus?: (message: string) => void;
	onError?: (message: string) => void;
}

export function useConnection(
	activeProfile: ComputedRef<StoredProfile | null>,
	options: UseConnectionOptions = {},
) {
	const connectionId = ref<string | null>(null);
	const tables = ref<TableInfo[]>([]);
	const activeTableName = ref<string | null>(null);
	const schema = ref<SchemaDefinition | null>(null);
	const isConnecting = ref(false);
	const isRefreshing = ref(false);
	const isOpening = ref(false);

	function resetConnection() {
		connectionId.value = null;
		tables.value = [];
		activeTableName.value = null;
		schema.value = null;
	}

	async function connectActiveProfile() {
		if (isConnecting.value) {
			return;
		}
		const profile = activeProfile.value;
		if (!profile) {
			options.onError?.("请先选择连接档案");
			return;
		}

		try {
			isConnecting.value = true;
			resetConnection();
			const connectProfile = toConnectProfile(profile);
			connectProfile.auth ??= { type: "none" };
			const response = unwrapEnvelope(await connectV1(connectProfile));
			connectionId.value = response.connectionId;
			options.onStatus?.(`已连接：${response.name}`);
			await refreshTables();
		} catch (error) {
			const message = error instanceof Error ? error.message : "连接失败";
			options.onError?.(message);
		} finally {
			isConnecting.value = false;
		}
	}

	async function refreshTables() {
		const id = connectionId.value;
		if (!id) {
			return;
		}
		if (isRefreshing.value) {
			return;
		}

		try {
			isRefreshing.value = true;
			const response = unwrapEnvelope(await listTablesV1(id));
			tables.value = response.tables;
		} catch (error) {
			const message = error instanceof Error ? error.message : "拉取表列表失败";
			options.onError?.(message);
		} finally {
			isRefreshing.value = false;
		}
	}

	async function openTable(name: string) {
		const id = connectionId.value;
		if (!id) {
			return;
		}
		if (isOpening.value) {
			return;
		}

		try {
			isOpening.value = true;
			activeTableName.value = name;
			schema.value = null;
			const handle = unwrapEnvelope(await openTableV1(id, name));
			schema.value = unwrapEnvelope(await getSchemaV1(handle.tableId));
		} catch (error) {
			const message = error instanceof Error ? error.message : "打开表失败";
			options.onError?.(message);
		} finally {
			isOpening.value = false;
		}
	}

	return {
		connectionId,
		tables,
		activeTableName,
		schema,
		isConnecting,
		isRefreshing,
		isOpening,
		connectActiveProfile,
		refreshTables,
		openTable,
		resetConnection,
	};
}

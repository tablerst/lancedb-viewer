<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { DataTableColumns, SelectOption } from "naive-ui";

import type { SchemaField } from "../ipc/v1";
import { scanV1, unwrapEnvelope } from "../lib/tauriClient";
import { formatCellValue, normalizeRow } from "../lib/formatters.ts";
import { useWorkspace } from "../composables/workspaceContext";

const {
	activeTableName,
	activeTableId,
	schema,
	isOpening,
	setError,
	setStatus,
	clearMessages,
} = useWorkspace();

const schemaColumns: DataTableColumns<SchemaField> = [
	{ title: "字段", key: "name" },
	{ title: "类型", key: "dataType" },
	{
		title: "Nullable",
		key: "nullable",
		render: (row) => (row.nullable ? "是" : "否"),
	},
];

const schemaData = computed(() => schema.value?.fields ?? []);
const allFieldNames = computed(() => schema.value?.fields.map((field) => field.name) ?? []);
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((name) => ({ label: name, value: name })),
);

const selectedColumns = ref<string[]>([]);
const limit = ref(50);
const offset = ref(0);
const filterExpression = ref("");

const isScanning = ref(false);
const scanError = ref("");
const dataRows = ref<unknown[]>([]);
const nextOffset = ref<number | null>(null);

const visibleColumns = computed(() =>
	selectedColumns.value.length ? selectedColumns.value : allFieldNames.value,
);
const hasActiveTable = computed(() => Boolean(activeTableId.value));

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() =>
	visibleColumns.value.map((name) => ({
		title: name,
		key: name,
		render: (row) => formatCellValue(row[name]),
	})),
);

const tableData = computed(() =>
	dataRows.value.map((row, index) => ({
		__rowId: `${offset.value + index}`,
		...normalizeRow(row),
	})),
);

watch(schema, () => {
	selectedColumns.value = allFieldNames.value;
});

watch(activeTableId, () => {
	offset.value = 0;
	dataRows.value = [];
	nextOffset.value = null;
	scanError.value = "";
	clearMessages();
	if (activeTableId.value) {
		void runScan();
	}
});

function selectAllColumns() {
	selectedColumns.value = allFieldNames.value;
}

function clearColumns() {
	selectedColumns.value = [];
}

async function runScan() {
	const tableId = activeTableId.value;
	if (!tableId || isScanning.value) {
		return;
	}

	try {
		isScanning.value = true;
		scanError.value = "";
		const response = unwrapEnvelope(
			await scanV1({
				tableId,
				format: "json",
				projection: selectedColumns.value.length ? selectedColumns.value : undefined,
				filter: filterExpression.value.trim() || undefined,
				limit: limit.value,
				offset: offset.value,
			}),
		);

		if (response.chunk.format !== "json") {
			scanError.value = "当前仅支持 JSON 数据块";
			return;
		}

		dataRows.value = response.chunk.rows;
		nextOffset.value = response.nextOffset ?? null;
		setStatus(`已加载 ${response.chunk.rows.length} 行数据`);
	} catch (error) {
		const message = error instanceof Error ? error.message : "扫描数据失败";
		scanError.value = message;
		setError(message);
	} finally {
		isScanning.value = false;
	}
}

function previousPage() {
	if (offset.value === 0 || isScanning.value) {
		return;
	}
	offset.value = Math.max(0, offset.value - limit.value);
	void runScan();
}

function nextPage() {
	if (nextOffset.value === null || isScanning.value) {
		return;
	}
	offset.value = nextOffset.value;
	void runScan();
}
</script>

<template>
	<NCard size="small" title="表详情" class="shadow-sm">
		<NEmpty v-if="!activeTableName" description="选择表以查看详情" />
		<div v-else class="space-y-4">
			<div class="flex flex-wrap items-center justify-between gap-2 text-sm text-slate-600">
				<span>当前表：{{ activeTableName }}</span>
				<span class="text-slate-400">•</span>
				<span v-if="isOpening">正在加载 schema…</span>
			</div>

			<NTabs type="line">
				<NTabPane name="schema" tab="Schema">
					<NDataTable
						size="small"
						:columns="schemaColumns"
						:data="schemaData"
						:bordered="false"
					/>
				</NTabPane>
				<NTabPane name="data" tab="数据浏览">
					<div class="space-y-3">
						<div class="grid gap-2 xl:grid-cols-5">
							<div class="space-y-1">
								<label class="text-xs text-slate-500">Limit</label>
								<NInputNumber
									v-model:value="limit"
									min="1"
									:disabled="isScanning || !hasActiveTable"
								/>
							</div>
							<div class="space-y-1">
								<label class="text-xs text-slate-500">Offset</label>
								<NInputNumber
									v-model:value="offset"
									min="0"
									:disabled="isScanning || !hasActiveTable"
								/>
							</div>
							<div class="space-y-1 xl:col-span-2">
								<label class="text-xs text-slate-500">过滤表达式</label>
								<NInput
									v-model:value="filterExpression"
									placeholder='only_if("id > 5")'
									:disabled="isScanning || !hasActiveTable"
								/>
							</div>
							<div class="flex flex-wrap items-end gap-2">
								<NButton
									type="primary"
									:loading="isScanning"
									:disabled="!hasActiveTable"
									@click="runScan"
								>
									查询
								</NButton>
								<NButton secondary :disabled="!hasActiveTable" @click="selectAllColumns">
									全部列
								</NButton>
								<NButton quaternary :disabled="!hasActiveTable" @click="clearColumns">
									取消投影
								</NButton>
							</div>
						</div>

						<div class="space-y-1">
							<label class="text-xs text-slate-500">列投影</label>
							<NSelect
								v-model:value="selectedColumns"
								:options="columnOptions"
								multiple
								clearable
								:disabled="isScanning || !hasActiveTable"
							/>
						</div>

						<NAlert v-if="scanError" type="error" :bordered="false">
							{{ scanError }}
						</NAlert>

						<NDataTable
							size="small"
							:columns="tableColumns"
							:data="tableData"
							:loading="isScanning"
							:bordered="false"
							:row-key="(row) => row.__rowId"
						/>

						<div class="flex flex-wrap items-center justify-between gap-2 text-xs text-slate-500">
							<NButton size="tiny" :disabled="offset === 0 || isScanning" @click="previousPage">
								上一页
							</NButton>
							<span>offset: {{ offset }} · limit: {{ limit }}</span>
							<NButton size="tiny" :disabled="nextOffset === null || isScanning" @click="nextPage">
								下一页
							</NButton>
						</div>
					</div>
				</NTabPane>
			</NTabs>
		</div>
	</NCard>
</template>

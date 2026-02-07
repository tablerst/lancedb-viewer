<script setup lang="ts">
import { open, save } from "@tauri-apps/plugin-dialog"
import type { SelectOption } from "naive-ui"
import { computed, inject, ref, watch } from "vue"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { DataFileFormatV1, WriteDataMode } from "../../ipc/v1"
import { exportDataV1, importDataV1, unwrapEnvelope } from "../../lib/tauriClient"
import { fileFormatOptions, TRIGGER_DATA_REFRESH_KEY, writeModeOptions } from "./explorerShared"

const { activeProfileId, activeTableId, schema, setError, setStatus, refreshSchema } =
	useWorkspace()

const triggerDataRefresh = inject(TRIGGER_DATA_REFRESH_KEY, () => {})

const hasActiveTable = computed(() => Boolean(activeTableId.value))
const allFieldNames = computed(() => schema.value?.fields.map((f) => f.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((n) => ({ label: n, value: n }))
)

// ── Import ─────────────────────────────────────────────

const importFormat = ref<DataFileFormatV1>("csv")
const importPath = ref("")
const importMode = ref<WriteDataMode>("append")
const importHasHeader = ref(true)
const importDelimiter = ref(",")
const { execute: execImport, isLoading: isImporting } = useCommand("导入失败")

const isCsvImport = computed(() => importFormat.value === "csv")

const fileDialogFilters: Record<DataFileFormatV1, { name: string; extensions: string[] }> = {
	csv: { name: "CSV", extensions: ["csv"] },
	parquet: { name: "Parquet", extensions: ["parquet"] },
	jsonl: { name: "JSONL", extensions: ["jsonl", "json"] },
}

function resolveDialogPath(value: string | string[] | null) {
	if (!value) {
		return ""
	}
	if (Array.isArray(value)) {
		return value[0] ?? ""
	}
	return value
}

async function selectImportFile() {
	const selection = await open({
		multiple: false,
		filters: [fileDialogFilters[importFormat.value]],
	})
	const path = resolveDialogPath(selection)
	if (path) {
		importPath.value = path
	}
}

async function submitImportData() {
	const profileId = activeProfileId.value
	const tableId = activeTableId.value
	const path = importPath.value.trim()
	if (!profileId || !tableId) {
		return
	}
	if (!path) {
		setError("请选择要导入的文件")
		return
	}
	const delimiter = importDelimiter.value.trim()
	await execImport(async () => {
		const response = unwrapEnvelope(
			await importDataV1({
				tableId,
				path,
				format: importFormat.value,
				mode: importMode.value,
				hasHeader: importHasHeader.value,
				delimiter: delimiter ? delimiter : undefined,
			})
		)
		setStatus(`已导入 ${response.rows} 行数据`)
		await refreshSchema(profileId)
		triggerDataRefresh()
	})
}

// ── Export ──────────────────────────────────────────────

const exportFormat = ref<DataFileFormatV1>("csv")
const exportPath = ref("")
const exportProjection = ref<string[]>([])
const exportFilter = ref("")
const exportLimit = ref<number | null>(1000)
const exportOffset = ref<number | null>(0)
const exportWithHeader = ref(true)
const exportDelimiter = ref(",")
const { execute: execExport, isLoading: isExporting } = useCommand("导出失败")

const isCsvExport = computed(() => exportFormat.value === "csv")

async function selectExportFile() {
	const selection = await save({
		filters: [fileDialogFilters[exportFormat.value]],
	})
	if (selection) {
		exportPath.value = selection
	}
}

async function submitExportData() {
	const tableId = activeTableId.value
	const path = exportPath.value.trim()
	if (!tableId) {
		return
	}
	if (!path) {
		setError("请选择导出文件路径")
		return
	}
	const limit = exportLimit.value && exportLimit.value > 0 ? exportLimit.value : undefined
	const offsetValue = exportOffset.value ?? undefined
	const delimiter = exportDelimiter.value.trim()
	await execExport(async () => {
		const response = unwrapEnvelope(
			await exportDataV1({
				tableId,
				path,
				format: exportFormat.value,
				projection: exportProjection.value.length ? exportProjection.value : undefined,
				filter: exportFilter.value.trim() || undefined,
				limit,
				offset: offsetValue,
				delimiter: delimiter ? delimiter : undefined,
				withHeader: exportWithHeader.value,
			})
		)
		setStatus(`已导出 ${response.rows} 行数据到 ${response.path}`)
	})
}

// ── Reset on table switch ──────────────────────────────

watch(activeTableId, () => {
	importFormat.value = "csv"
	importPath.value = ""
	importMode.value = "append"
	importHasHeader.value = true
	importDelimiter.value = ","
	isImporting.value = false
	exportFormat.value = "csv"
	exportPath.value = ""
	exportProjection.value = []
	exportFilter.value = ""
	exportLimit.value = 1000
	exportOffset.value = 0
	exportWithHeader.value = true
	exportDelimiter.value = ","
	isExporting.value = false
})
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="导入数据" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">文件路径</label>
					<div class="flex items-center gap-2">
						<NInput
							v-model:value="importPath"
							placeholder="选择要导入的文件"
							:disabled="isImporting"
						/>
						<NButton secondary :disabled="isImporting" @click="selectImportFile">
							选择文件
						</NButton>
					</div>
				</div>
				<div class="xl:col-span-3 grid grid-cols-2 gap-3">
					<div>
						<label class="text-sm font-medium text-slate-600">格式</label>
						<NSelect
							v-model:value="importFormat"
							:options="fileFormatOptions"
							:disabled="isImporting"
						/>
					</div>
					<div>
						<label class="text-sm font-medium text-slate-600">写入模式</label>
						<NSelect
							v-model:value="importMode"
							:options="writeModeOptions"
							:disabled="isImporting"
						/>
					</div>
				</div>
			</div>
			<div class="mt-3 grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">CSV 选项</label>
					<div class="flex items-center gap-3">
						<NCheckbox
							v-model:checked="importHasHeader"
							:disabled="!isCsvImport || isImporting"
						>
							包含表头
						</NCheckbox>
						<NInput
							v-model:value="importDelimiter"
							placeholder=","
							:disabled="!isCsvImport || isImporting"
							:maxlength="1"
							class="w-24"
						/>
					</div>
				</div>
				<div class="xl:col-span-3 flex items-end justify-end">
					<NButton
						type="primary"
						:loading="isImporting"
						:disabled="!hasActiveTable"
						@click="submitImportData"
					>
						开始导入
					</NButton>
				</div>
			</div>
		</NCard>

		<NCard size="small" title="导出数据" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">导出路径</label>
					<div class="flex items-center gap-2">
						<NInput
							v-model:value="exportPath"
							placeholder="选择导出文件位置"
							:disabled="isExporting"
						/>
						<NButton secondary :disabled="isExporting" @click="selectExportFile">
							选择路径
						</NButton>
					</div>
				</div>
				<div class="xl:col-span-3 grid grid-cols-2 gap-3">
					<div>
						<label class="text-sm font-medium text-slate-600">格式</label>
						<NSelect
							v-model:value="exportFormat"
							:options="fileFormatOptions"
							:disabled="isExporting"
						/>
					</div>
					<div>
						<label class="text-sm font-medium text-slate-600">Filter</label>
						<NInput
							v-model:value="exportFilter"
							placeholder="id > 10"
							:disabled="isExporting"
						/>
					</div>
				</div>
			</div>
			<div class="mt-3 grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">列投影</label>
					<NSelect
						v-model:value="exportProjection"
						:options="columnOptions"
						multiple
						clearable
						:disabled="isExporting"
					/>
				</div>
				<div class="xl:col-span-3 grid grid-cols-3 gap-3">
					<div>
						<label class="text-sm font-medium text-slate-600">Limit</label>
						<NInputNumber
							v-model:value="exportLimit"
							:min="1"
							:disabled="isExporting"
						/>
					</div>
					<div>
						<label class="text-sm font-medium text-slate-600">Offset</label>
						<NInputNumber
							v-model:value="exportOffset"
							:min="0"
							:disabled="isExporting"
						/>
					</div>
					<div class="flex items-end justify-end">
						<NButton
							type="primary"
							:loading="isExporting"
							:disabled="!hasActiveTable"
							@click="submitExportData"
						>
							开始导出
						</NButton>
					</div>
				</div>
			</div>
			<div class="mt-3 flex flex-wrap items-center gap-3">
				<NCheckbox
					v-model:checked="exportWithHeader"
					:disabled="!isCsvExport || isExporting"
				>
					CSV 输出表头
				</NCheckbox>
				<NInput
					v-model:value="exportDelimiter"
					placeholder=","
					:disabled="!isCsvExport || isExporting"
					:maxlength="1"
					class="w-24"
				/>
			</div>
		</NCard>
	</div>
</template>

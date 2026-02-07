<script setup lang="ts">
import { save } from "@tauri-apps/plugin-dialog"
import type { SelectOption } from "naive-ui"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { DataFileFormatV1 } from "../../ipc/v1"
import { exportDataV1, unwrapEnvelope } from "../../lib/tauriClient"
import { fileFormatOptions } from "./explorerShared"

const props = defineProps<{
	show: boolean
}>()

const emit = defineEmits<(e: "update:show", value: boolean) => void>()

const { activeTableId, schema, setError, setStatus } = useWorkspace()
const hasActiveTable = computed(() => Boolean(activeTableId.value))
const allFieldNames = computed(() => schema.value?.fields.map((f) => f.name) ?? [])
const columnOptions = computed<SelectOption[]>(() =>
	allFieldNames.value.map((n) => ({ label: n, value: n }))
)

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

const fileDialogFilters: Record<DataFileFormatV1, { name: string; extensions: string[] }> = {
	csv: { name: "CSV", extensions: ["csv"] },
	parquet: { name: "Parquet", extensions: ["parquet"] },
	jsonl: { name: "JSONL", extensions: ["jsonl", "json"] },
}

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
	if (!tableId) return
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
		close()
	})
}

function close() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(visible) => {
		if (visible) {
			exportFormat.value = "csv"
			exportPath.value = ""
			exportProjection.value = []
			exportFilter.value = ""
			exportLimit.value = 1000
			exportOffset.value = 0
			exportWithHeader.value = true
			exportDelimiter.value = ","
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		:mask-closable="!isExporting"
		:close-on-esc="!isExporting"
		@update:show="emit('update:show', $event)"
	>
		<NCard
			size="small"
			title="导出数据"
			class="w-[620px] max-w-[calc(100vw-40px)]"
			:closable="!isExporting"
			:bordered="false"
			@close="close"
		>
			<div class="space-y-3">
				<div>
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
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="text-sm font-medium text-slate-600">格式</label>
						<NSelect
							v-model:value="exportFormat"
							:options="fileFormatOptions"
							:disabled="isExporting"
						/>
					</div>
					<div>
						<label class="text-sm font-medium text-slate-600">过滤条件</label>
						<NInput
							v-model:value="exportFilter"
							placeholder="id > 10"
							:disabled="isExporting"
						/>
					</div>
				</div>
				<div>
					<label class="text-sm font-medium text-slate-600">列投影</label>
					<NSelect
						v-model:value="exportProjection"
						:options="columnOptions"
						multiple
						clearable
						:disabled="isExporting"
					/>
				</div>
				<div class="grid grid-cols-3 gap-3">
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
					<div class="space-y-1">
						<NCheckbox
							v-model:checked="exportWithHeader"
							:disabled="!isCsvExport || isExporting"
						>
							CSV 表头
						</NCheckbox>
						<NInput
							v-model:value="exportDelimiter"
							placeholder=","
							:disabled="!isCsvExport || isExporting"
							:maxlength="1"
							class="w-24"
						/>
					</div>
				</div>
				<div class="flex items-center justify-end gap-2">
					<NButton quaternary :disabled="isExporting" @click="close">
						取消
					</NButton>
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
		</NCard>
	</NModal>
</template>

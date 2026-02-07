<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import type { DataFileFormatV1, WriteDataMode } from "../../ipc/v1"
import { importDataV1, unwrapEnvelope } from "../../lib/tauriClient"
import { fileFormatOptions, TRIGGER_DATA_REFRESH_KEY, writeModeOptions } from "./explorerShared"

const props = defineProps<{
	show: boolean
}>()

const emit = defineEmits<(e: "update:show", value: boolean) => void>()

const { activeProfileId, activeTableId, setError, setStatus, refreshSchema } = useWorkspace()
const triggerDataRefresh = inject(TRIGGER_DATA_REFRESH_KEY, () => {})

const hasActiveTable = computed(() => Boolean(activeTableId.value))

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
	if (!value) return ""
	if (Array.isArray(value)) return value[0] ?? ""
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
	if (!profileId || !tableId) return
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
			importFormat.value = "csv"
			importPath.value = ""
			importMode.value = "append"
			importHasHeader.value = true
			importDelimiter.value = ","
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		:mask-closable="!isImporting"
		:close-on-esc="!isImporting"
		@update:show="emit('update:show', $event)"
	>
		<NCard
			size="small"
			title="导入数据"
			class="w-[560px] max-w-[calc(100vw-40px)]"
			:closable="!isImporting"
			:bordered="false"
			@close="close"
		>
			<div class="space-y-3">
				<div>
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
				<div class="grid grid-cols-2 gap-3">
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
				<div>
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
				<div class="flex items-center justify-end gap-2">
					<NButton quaternary :disabled="isImporting" @click="close">
						取消
					</NButton>
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
	</NModal>
</template>

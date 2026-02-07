<script setup lang="ts">
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import { renameTableV1, unwrapEnvelope } from "../../lib/tauriClient"

const props = defineProps<{
	show: boolean
	tableName: string | null
}>()

const emit = defineEmits<(e: "update:show", value: boolean) => void>()

const {
	activeProfileId,
	connectionId,
	activeTableName,
	setError,
	setStatus,
	refreshTables,
	openTable,
} = useWorkspace()

const { execute: execRenameTable, isLoading: isRenamingTable } = useCommand("重命名表失败")
const renameTargetName = ref("")

async function submitRenameTable() {
	const profileId = activeProfileId.value
	const currentConnectionId = connectionId.value
	const tableName = props.tableName ?? activeTableName.value
	if (!profileId || !currentConnectionId || !tableName) return
	const newTableName = renameTargetName.value.trim()
	if (!newTableName) {
		setError("请输入新表名")
		return
	}
	if (newTableName === tableName) {
		setError("新表名不能与当前表名相同")
		return
	}
	await execRenameTable(async () => {
		const response = unwrapEnvelope(
			await renameTableV1({
				connectionId: currentConnectionId,
				tableName,
				newTableName,
			})
		)
		setStatus(`已重命名为 ${response.newTableName}`)
		renameTargetName.value = ""
		close()
		await refreshTables(profileId)
		await openTable(profileId, response.newTableName)
	})
}

function close() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(visible) => {
		if (visible) {
			renameTargetName.value = props.tableName ?? ""
		} else {
			renameTargetName.value = ""
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		:mask-closable="!isRenamingTable"
		:close-on-esc="!isRenamingTable"
		@update:show="emit('update:show', $event)"
	>
		<NCard
			size="small"
			title="重命名表"
			class="w-[420px]"
			:closable="!isRenamingTable"
			:bordered="false"
			@close="close"
		>
			<div class="space-y-3">
				<div class="text-xs text-slate-500">
					当前表：{{ tableName ?? "—" }}
				</div>
				<NInput
					v-model:value="renameTargetName"
					placeholder="new_table_name"
					:disabled="isRenamingTable"
				/>
				<div class="flex items-center justify-end gap-2">
					<NButton quaternary :disabled="isRenamingTable" @click="close">
						取消
					</NButton>
					<NButton
						type="primary"
						:loading="isRenamingTable"
						@click="submitRenameTable"
					>
						确认重命名
					</NButton>
				</div>
				<div class="text-[11px] text-slate-400">
					仅 LanceDB Cloud 支持重命名；本地连接将提示不支持。
				</div>
			</div>
		</NCard>
	</NModal>
</template>

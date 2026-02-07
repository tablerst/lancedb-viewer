<script setup lang="ts">
import { computed, ref } from "vue"
import { useCommand } from "../../composables/useCommand"
import { useWorkspace } from "../../composables/workspaceContext"
import { optimizeTableV1, unwrapEnvelope } from "../../lib/tauriClient"

const { activeTableId, setError, setStatus } = useWorkspace()

const hasActiveTable = computed(() => Boolean(activeTableId.value))

const compactTargetRows = ref<number | null>(1_000_000)
const vacuumOlderThanDays = ref<number | null>(7)
const { execute: execCompact, isLoading: isCompacting } = useCommand("Compact 失败")
const { execute: execVacuum, isLoading: isVacuuming } = useCommand("Vacuum 失败")

async function submitCompactTable() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}
	const targetRows = compactTargetRows.value
	if (targetRows !== null && targetRows <= 0) {
		setError("目标片段行数必须大于 0")
		return
	}
	await execCompact(async () => {
		const response = unwrapEnvelope(
			await optimizeTableV1({
				tableId,
				action: "compact",
				targetRowsPerFragment: targetRows ?? undefined,
			})
		)
		setStatus(response.summary || "Compact 已完成")
	})
}

async function submitVacuumTable() {
	const tableId = activeTableId.value
	if (!tableId) {
		return
	}
	const olderThanDays = vacuumOlderThanDays.value
	if (olderThanDays !== null && olderThanDays < 0) {
		setError("保留天数不能为负数")
		return
	}
	await execVacuum(async () => {
		const response = unwrapEnvelope(
			await optimizeTableV1({
				tableId,
				action: "vacuum",
				olderThanDays: olderThanDays ?? undefined,
			})
		)
		setStatus(response.summary || "Vacuum 已完成")
	})
}
</script>

<template>
	<div class="space-y-4">
		<NCard size="small" title="Compact（合并数据文件）" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">目标片段行数</label>
					<NInputNumber
						v-model:value="compactTargetRows"
						:min="1"
						:disabled="!hasActiveTable || isCompacting"
					/>
				</div>
				<div class="xl:col-span-3 flex items-end justify-end">
					<NButton
						type="primary"
						:loading="isCompacting"
						:disabled="!hasActiveTable"
						@click="submitCompactTable"
					>
						执行 Compact
					</NButton>
				</div>
			</div>
			<div class="mt-2 text-xs text-slate-400">
				Compact 会合并小文件并重写片段，过程可能耗时。
			</div>
		</NCard>

		<NCard size="small" title="Vacuum（清理旧版本）" class="shadow-sm">
			<div class="grid gap-3 xl:grid-cols-6">
				<div class="xl:col-span-3">
					<label class="text-sm font-medium text-slate-600">保留天数</label>
					<NInputNumber
						v-model:value="vacuumOlderThanDays"
						:min="0"
						:disabled="!hasActiveTable || isVacuuming"
					/>
				</div>
				<div class="xl:col-span-3 flex items-end justify-end">
					<NPopconfirm
						positive-text="执行"
						negative-text="取消"
						@positive-click="submitVacuumTable"
					>
						<template #trigger>
							<NButton
								type="primary"
								:loading="isVacuuming"
								:disabled="!hasActiveTable"
							>
								执行 Vacuum
							</NButton>
						</template>
						将清理超过指定天数的旧版本与未引用文件，确定继续吗？
					</NPopconfirm>
				</div>
			</div>
			<div class="mt-2 text-xs text-slate-400">
				建议在低峰期执行；部分远程后端可能不支持该操作。
			</div>
		</NCard>
	</div>
</template>

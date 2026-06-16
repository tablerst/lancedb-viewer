<script setup lang="ts">
/**
 * BatchUpdateDialog — modal dialog for batch updating rows with expressions.
 */

interface UpdateDraft {
	column: string
	expr: string
}

const props = defineProps<{
	show: boolean
	loading: boolean
}>()

const emit = defineEmits<{
	(e: "update:show", value: boolean): void
	(e: "submit", filter: string | undefined, updates: Array<{ column: string; expr: string }>): void
}>()

const filterExpr = ref("")
const updateColumns = ref<UpdateDraft[]>([{ column: "", expr: "" }])

function addColumn() {
	updateColumns.value = [...updateColumns.value, { column: "", expr: "" }]
}

function removeColumn(index: number) {
	updateColumns.value = updateColumns.value.filter((_, idx) => idx !== index)
}

function handleSubmit() {
	const updates = updateColumns.value
		.map((item) => ({ column: item.column.trim(), expr: item.expr.trim() }))
		.filter((item) => item.column && item.expr)
	if (!updates.length) return
	emit("submit", filterExpr.value.trim() || undefined, updates)
}

function handleClose() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(v) => {
		if (v) {
			filterExpr.value = ""
			updateColumns.value = [{ column: "", expr: "" }]
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		preset="card"
		title="批量更新数据"
		:bordered="false"
		style="width: 620px; max-width: 90vw"
		@update:show="handleClose"
	>
		<div class="space-y-4">
			<div>
				<label
					for="batch-update-filter"
					class="mb-1 block text-sm font-medium text-slate-600"
				>
					过滤条件（可选，不填则更新所有行）
				</label>
				<NInput
					v-model:value="filterExpr"
					placeholder="id = 1"
					size="small"
					input-id="batch-update-filter"
				/>
			</div>

			<div>
				<div class="mb-2 flex items-center justify-between">
					<label class="text-sm font-medium text-slate-600">更新列</label>
					<NButton size="tiny" secondary @click="addColumn">+ 添加列</NButton>
				</div>
				<div class="space-y-2">
					<div
						v-for="(item, index) in updateColumns"
						:key="`update-${index}`"
						class="grid gap-2 sm:grid-cols-[minmax(8rem,10rem)_minmax(0,1fr)_auto] sm:items-center"
					>
						<NInput
							v-model:value="item.column"
							placeholder="列名"
							size="small"
							class="min-w-0"
							:input-id="`batch-update-column-${index}`"
							:aria-label="`第 ${index + 1} 个更新列名`"
						/>
						<NInput
							v-model:value="item.expr"
							placeholder="表达式，例如 text || '_v2'"
							size="small"
							class="min-w-0"
							:input-id="`batch-update-expression-${index}`"
							:aria-label="`第 ${index + 1} 个更新表达式`"
						/>
						<NButton
							v-if="updateColumns.length > 1"
							size="tiny"
							quaternary
							class="justify-self-start"
							@click="removeColumn(index)"
						>
							移除
						</NButton>
					</div>
				</div>
			</div>
		</div>
		<template #action>
			<div class="flex justify-end gap-2">
				<NButton @click="handleClose">取消</NButton>
				<NButton type="primary" :loading="loading" @click="handleSubmit">
					提交更新
				</NButton>
			</div>
		</template>
	</NModal>
</template>

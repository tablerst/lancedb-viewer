<script setup lang="ts">
/**
 * BatchDeleteDialog — modal dialog for batch deleting rows by filter expression.
 */

const props = defineProps<{
	show: boolean
	loading: boolean
}>()

const emit = defineEmits<{
	(e: "update:show", value: boolean): void
	(e: "submit", filter: string): void
}>()

const filterExpr = ref("")

function handleSubmit() {
	const f = filterExpr.value.trim()
	if (!f) return
	emit("submit", f)
}

function handleClose() {
	emit("update:show", false)
}

watch(
	() => props.show,
	(v) => {
		if (v) {
			filterExpr.value = ""
		}
	}
)
</script>

<template>
	<NModal
		:show="show"
		preset="card"
		title="批量删除数据"
		:bordered="false"
		style="width: 500px; max-width: 90vw"
		@update:show="handleClose"
	>
		<div class="space-y-4">
			<NAlert type="warning" :bordered="false">
				此操作将永久删除符合条件的数据，请谨慎操作。
			</NAlert>
			<div>
				<label class="mb-1 block text-sm font-medium text-slate-600">删除条件</label>
				<NInput
					v-model:value="filterExpr"
					placeholder="id = 1"
					size="small"
				/>
				<p class="mt-1 text-xs text-slate-400">
					输入 SQL 风格的过滤表达式来指定要删除的行
				</p>
			</div>
		</div>
		<template #action>
			<div class="flex justify-end gap-2">
				<NButton @click="handleClose">取消</NButton>
				<NPopconfirm
					positive-text="确认删除"
					negative-text="取消"
					@positive-click="handleSubmit"
				>
					<template #trigger>
						<NButton type="error" :loading="loading" :disabled="!filterExpr.trim()">
							删除数据
						</NButton>
					</template>
					确定删除符合条件的数据吗？此操作不可撤销。
				</NPopconfirm>
			</div>
		</template>
	</NModal>
</template>

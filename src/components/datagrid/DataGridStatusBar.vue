<script setup lang="ts">
/**
 * DataGridStatusBar — bottom status bar with pagination, row count, timing, and dirty count.
 */

defineProps<{
	page: number
	pageSize: number
	pageCount: number
	rowCount: number
	dirtyCount: number
	loading: boolean
	disabled: boolean
	loadTimeMs: number | null
	hasNextPage: boolean
}>()

const emit = defineEmits<{
	(e: "update:page", page: number): void
	(e: "update:page-size", size: number): void
}>()

const pageSizes = [10, 20, 50, 100, 200]

function prevPage(page: number) {
	if (page > 1) emit("update:page", page - 1)
}

function nextPage(page: number, pageCount: number) {
	if (page < pageCount) emit("update:page", page + 1)
}
</script>

<template>
	<div class="datagrid-statusbar">
		<div class="datagrid-statusbar-left">
			<span class="datagrid-statusbar-info">
				已加载 {{ rowCount }} 行
				<template v-if="hasNextPage">（还有更多）</template>
			</span>
			<span v-if="loadTimeMs !== null" class="datagrid-statusbar-info datagrid-statusbar-time">
				{{ (loadTimeMs / 1000).toFixed(2) }}s
			</span>
			<span v-if="dirtyCount > 0" class="datagrid-statusbar-dirty">
				{{ dirtyCount }} 项待保存
			</span>
		</div>

		<div class="datagrid-statusbar-right">
			<select
				class="datagrid-statusbar-select"
				:value="pageSize"
				@change="emit('update:page-size', Number(($event.target as HTMLSelectElement).value))"
			>
				<option v-for="size in pageSizes" :key="size" :value="size">
					{{ size }} / page
				</option>
			</select>

			<div class="datagrid-statusbar-pager">
				<button
					class="datagrid-statusbar-page-btn"
					:disabled="page <= 1 || loading || disabled"
					aria-label="上一页数据"
					title="上一页数据"
					@click="prevPage(page)"
				>
					‹
				</button>
				<span class="datagrid-statusbar-page-num">{{ page }}</span>
				<button
					class="datagrid-statusbar-page-btn"
					:disabled="page >= pageCount || loading || disabled"
					aria-label="下一页数据"
					title="下一页数据"
					@click="nextPage(page, pageCount)"
				>
					›
				</button>
			</div>
		</div>
	</div>
</template>

<style scoped>
.datagrid-statusbar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	height: 32px;
	padding: 0 12px;
	background: var(--app-surface-panel-muted);
	border-top: 1px solid var(--app-rule);
	font-size: 12px;
	color: var(--app-muted);
	gap: 12px;
}

.datagrid-statusbar-left,
.datagrid-statusbar-right {
	display: flex;
	align-items: center;
	gap: 12px;
}

.datagrid-statusbar-info {
	white-space: nowrap;
}

.datagrid-statusbar-time {
	color: var(--app-subtle);
}

.datagrid-statusbar-dirty {
	color: #d97706;
	font-weight: 600;
}

.datagrid-statusbar-select {
	height: 22px;
	padding: 0 4px;
	border: 1px solid var(--app-rule);
	border-radius: 4px;
	font-size: 12px;
	background: var(--app-control);
	color: var(--app-ink);
	cursor: pointer;
}

.datagrid-statusbar-pager {
	display: flex;
	align-items: center;
	gap: 4px;
}

.datagrid-statusbar-page-btn {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 22px;
	height: 22px;
	border: 1px solid var(--app-rule);
	border-radius: 4px;
	background: var(--app-control);
	color: var(--app-muted);
	font-size: 14px;
	font-weight: 600;
	cursor: pointer;
	line-height: 1;
}

.datagrid-statusbar-page-btn:hover:not(:disabled) {
	background: var(--app-control-hover);
	border-color: var(--app-rule-strong);
}

.datagrid-statusbar-page-btn:disabled {
	color: var(--app-subtle);
	cursor: not-allowed;
	opacity: 0.5;
}

.datagrid-statusbar-page-num {
	min-width: 20px;
	text-align: center;
	font-weight: 600;
	color: var(--app-ink);
	font-variant-numeric: tabular-nums;
}
</style>

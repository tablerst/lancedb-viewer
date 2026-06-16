<script setup lang="ts">
/**
 * DataGridToolbar — toolbar above the data grid.
 * Left side: refresh, advanced filter toggle, active filter badge.
 * Right side: add row, batch ops, save/discard, export.
 */
import {
	Download,
	FileUp,
	Pencil,
	Plus,
	RefreshCcw,
	Save,
	SlidersHorizontal,
	Trash2,
	Undo2,
} from "lucide-vue-next"

defineProps<{
	loading: boolean
	disabled: boolean
	showAdvancedFilter: boolean
	hasChanges: boolean
	dirtyCount: number
	activeFilterCount: number
}>()

const emit = defineEmits<{
	(e: "refresh"): void
	(e: "toggle-advanced-filter"): void
	(e: "add-row"): void
	(e: "save"): void
	(e: "discard"): void
	(e: "export"): void
	(e: "open-batch-write"): void
	(e: "open-batch-update"): void
	(e: "open-batch-delete"): void
}>()
</script>

<template>
	<div class="datagrid-toolbar">
		<div class="datagrid-toolbar-left">
			<button
				class="datagrid-toolbar-btn"
				:class="{ 'datagrid-toolbar-btn--loading': loading }"
				title="刷新数据 (F5)"
				aria-label="刷新数据"
				:disabled="disabled || loading"
				@click="emit('refresh')"
			>
				<RefreshCcw class="h-4 w-4" :class="{ 'animate-spin': loading }" />
			</button>

			<div class="datagrid-toolbar-sep" />

			<button
				class="datagrid-toolbar-btn"
				:class="{ 'datagrid-toolbar-btn--active': showAdvancedFilter }"
				title="高级筛选 (Ctrl+Shift+F)"
				aria-label="高级筛选"
				:disabled="disabled"
				@click="emit('toggle-advanced-filter')"
			>
				<SlidersHorizontal class="h-4 w-4" />
			</button>

			<span v-if="activeFilterCount > 0" class="datagrid-toolbar-filter-tag">
				{{ activeFilterCount }} 个过滤器
			</span>
		</div>

		<div class="datagrid-toolbar-right">
			<!-- Inline editing actions -->
			<button
				class="datagrid-toolbar-btn"
				title="添加行 (Ctrl+N)"
				aria-label="添加行"
				:disabled="disabled"
				@click="emit('add-row')"
			>
				<Plus class="h-4 w-4" />
			</button>

			<button
				v-if="hasChanges"
				class="datagrid-toolbar-btn datagrid-toolbar-btn--warning"
				title="放弃更改"
				aria-label="放弃更改"
				@click="emit('discard')"
			>
				<Undo2 class="h-4 w-4" />
			</button>

			<button
				class="datagrid-toolbar-btn"
				:class="{ 'datagrid-toolbar-btn--primary': hasChanges }"
				title="保存更改 (Ctrl+S)"
				aria-label="保存更改"
				:disabled="disabled || !hasChanges"
				@click="emit('save')"
			>
				<Save class="h-4 w-4" />
				<span v-if="dirtyCount > 0" class="datagrid-toolbar-badge datagrid-toolbar-badge--amber">
					{{ dirtyCount }}
				</span>
			</button>

			<div class="datagrid-toolbar-sep" />

			<!-- Batch operation buttons -->
			<button
				class="datagrid-toolbar-btn"
				title="批量写入数据"
				aria-label="批量写入数据"
				:disabled="disabled"
				@click="emit('open-batch-write')"
			>
				<FileUp class="h-4 w-4" />
			</button>

			<button
				class="datagrid-toolbar-btn"
				title="批量更新数据"
				aria-label="批量更新数据"
				:disabled="disabled"
				@click="emit('open-batch-update')"
			>
				<Pencil class="h-4 w-4" />
			</button>

			<button
				class="datagrid-toolbar-btn"
				title="批量删除数据"
				aria-label="批量删除数据"
				:disabled="disabled"
				@click="emit('open-batch-delete')"
			>
				<Trash2 class="h-4 w-4" />
			</button>

			<div class="datagrid-toolbar-sep" />

			<button
				class="datagrid-toolbar-btn"
				title="导出数据"
				aria-label="导出数据"
				:disabled="disabled"
				@click="emit('export')"
			>
				<Download class="h-4 w-4" />
			</button>
		</div>
	</div>
</template>

<style scoped>
.datagrid-toolbar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	height: 40px;
	padding: 0 8px;
	background: white;
	border-bottom: 1px solid #e2e8f0;
	gap: 2px;
}

.datagrid-toolbar-left,
.datagrid-toolbar-right {
	display: flex;
	align-items: center;
	gap: 2px;
}

.datagrid-toolbar-btn {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 4px;
	height: 30px;
	min-width: 30px;
	padding: 0 6px;
	border: none;
	border-radius: 5px;
	background: transparent;
	color: #475569;
	font-size: 13px;
	cursor: pointer;
	position: relative;
	transition: background 0.15s, color 0.15s;
}

.datagrid-toolbar-btn:hover:not(:disabled) {
	background: #f1f5f9;
	color: #1e293b;
}

.datagrid-toolbar-btn:disabled {
	color: #cbd5e1;
	cursor: not-allowed;
}

.datagrid-toolbar-btn--active {
	background: #e0f2fe;
	color: #0284c7;
}

.datagrid-toolbar-btn--active:hover:not(:disabled) {
	background: #bae6fd;
}

.datagrid-toolbar-btn--primary {
	background: #38bdf8;
	color: white;
}

.datagrid-toolbar-btn--primary:hover:not(:disabled) {
	background: #0ea5e9;
}

.datagrid-toolbar-btn--warning {
	color: #d97706;
}

.datagrid-toolbar-btn--warning:hover:not(:disabled) {
	background: #fffbeb;
}

.datagrid-toolbar-btn--loading {
	color: #38bdf8;
}

.datagrid-toolbar-sep {
	width: 1px;
	height: 20px;
	background: #e2e8f0;
	margin: 0 4px;
}

.datagrid-toolbar-badge {
	font-size: 10px;
	line-height: 1;
	padding: 1px 4px;
	border-radius: 8px;
	background: #0ea5e9;
	color: white;
	font-weight: 600;
}

.datagrid-toolbar-badge--amber {
	background: #f59e0b;
}

.datagrid-toolbar-filter-tag {
	font-size: 11px;
	line-height: 1;
	padding: 3px 8px;
	border-radius: 10px;
	background: #e0f2fe;
	color: #0284c7;
	font-weight: 500;
	white-space: nowrap;
}
</style>

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
			<div class="datagrid-toolbar-group" aria-label="视图操作">
				<span class="datagrid-toolbar-group-label">视图</span>
				<button
					class="datagrid-toolbar-btn datagrid-toolbar-btn--with-label"
					:class="{ 'datagrid-toolbar-btn--loading': loading }"
					title="刷新数据 (F5)"
					aria-label="刷新数据"
					:disabled="disabled || loading"
					@click="emit('refresh')"
				>
					<RefreshCcw class="h-4 w-4" :class="{ 'animate-spin': loading }" />
					<span>刷新</span>
				</button>

				<button
					class="datagrid-toolbar-btn datagrid-toolbar-btn--with-label"
					:class="{ 'datagrid-toolbar-btn--active': showAdvancedFilter }"
					title="高级筛选 (Ctrl+Shift+F)"
					aria-label="高级筛选"
					:disabled="disabled"
					@click="emit('toggle-advanced-filter')"
				>
					<SlidersHorizontal class="h-4 w-4" />
					<span>筛选</span>
				</button>
			</div>

			<span v-if="activeFilterCount > 0" class="datagrid-toolbar-filter-tag">
				{{ activeFilterCount }} 个过滤器
			</span>
		</div>

		<div class="datagrid-toolbar-right">
			<div class="datagrid-toolbar-group" aria-label="编辑操作">
				<span class="datagrid-toolbar-group-label">编辑</span>
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
					<span v-if="dirtyCount > 0" class="datagrid-toolbar-badge datagrid-toolbar-badge--dirty">
						{{ dirtyCount }}
					</span>
				</button>
			</div>

			<div class="datagrid-toolbar-group" aria-label="批量操作">
				<span class="datagrid-toolbar-group-label">批量</span>
				<button
					class="datagrid-toolbar-btn datagrid-toolbar-btn--with-label"
					title="批量写入数据"
					aria-label="批量写入数据"
					:disabled="disabled"
					@click="emit('open-batch-write')"
				>
					<FileUp class="h-4 w-4" />
					<span>写入</span>
				</button>

				<button
					class="datagrid-toolbar-btn datagrid-toolbar-btn--with-label"
					title="批量更新数据"
					aria-label="批量更新数据"
					:disabled="disabled"
					@click="emit('open-batch-update')"
				>
					<Pencil class="h-4 w-4" />
					<span>更新</span>
				</button>

				<button
					class="datagrid-toolbar-btn datagrid-toolbar-btn--danger datagrid-toolbar-btn--with-label"
					title="批量删除数据"
					aria-label="批量删除数据"
					:disabled="disabled"
					@click="emit('open-batch-delete')"
				>
					<Trash2 class="h-4 w-4" />
					<span>删除</span>
				</button>
			</div>

			<div class="datagrid-toolbar-group" aria-label="输出操作">
				<span class="datagrid-toolbar-group-label">输出</span>
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
	</div>
</template>

<style scoped>
.datagrid-toolbar {
	display: flex;
	align-items: center;
	align-content: flex-start;
	flex-wrap: nowrap;
	justify-content: space-between;
	min-height: 46px;
	padding: 6px 8px;
	background: var(--app-surface-panel);
	border-bottom: 1px solid var(--app-rule);
	gap: 8px;
	overflow-x: auto;
	scrollbar-width: thin;
}

.datagrid-toolbar-left,
.datagrid-toolbar-right {
	display: flex;
	align-items: center;
	flex: 0 0 auto;
	flex-wrap: nowrap;
	gap: 6px;
	min-width: 0;
}

.datagrid-toolbar-left {
	min-width: max-content;
}

.datagrid-toolbar-right {
	min-width: max-content;
	justify-content: flex-start;
}

.datagrid-toolbar-group {
	display: inline-flex;
	align-items: center;
	gap: 4px;
	padding: 2px;
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
	background: var(--app-surface-elevated);
	white-space: nowrap;
}

.datagrid-toolbar-group-label {
	padding: 0 5px;
	color: var(--app-subtle);
	font-size: 10px;
	font-weight: 600;
	line-height: 22px;
}

.datagrid-toolbar-btn {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 4px;
	height: 30px;
	min-width: 30px;
	padding: 0 6px;
	border: 1px solid transparent;
	border-radius: var(--app-radius-sm);
	background: transparent;
	color: var(--app-muted);
	font-size: 13px;
	cursor: pointer;
	position: relative;
	transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.datagrid-toolbar-btn--with-label {
	min-width: 52px;
}

.datagrid-toolbar-btn:hover:not(:disabled) {
	background: var(--app-control-hover);
	border-color: var(--app-rule);
	color: var(--app-ink);
}

.datagrid-toolbar-btn:disabled {
	color: var(--app-subtle);
	cursor: not-allowed;
	opacity: 0.45;
}

.datagrid-toolbar-btn--active {
	border-color: color-mix(in srgb, var(--app-accent) 32%, var(--app-rule));
	background: var(--app-accent-soft);
	color: var(--app-accent-strong);
}

.datagrid-toolbar-btn--active:hover:not(:disabled) {
	background: var(--app-accent-soft);
}

.datagrid-toolbar-btn--primary {
	border-color: var(--app-accent);
	background: var(--app-accent);
	color: white;
}

.datagrid-toolbar-btn--primary:hover:not(:disabled) {
	background: var(--app-accent-strong);
}

.datagrid-toolbar-btn--warning {
	color: var(--app-warning);
}

.datagrid-toolbar-btn--warning:hover:not(:disabled) {
	background: var(--app-warning-soft);
	border-color: color-mix(in srgb, var(--app-warning) 34%, var(--app-rule));
}

.datagrid-toolbar-btn--danger {
	color: var(--app-danger);
}

.datagrid-toolbar-btn--danger:hover:not(:disabled) {
	background: var(--app-danger-soft);
	border-color: color-mix(in srgb, var(--app-danger) 34%, var(--app-rule));
}

.datagrid-toolbar-btn--loading {
	color: var(--app-accent);
}

.datagrid-toolbar-sep {
	width: 1px;
	height: 20px;
	background: var(--app-rule);
	margin: 0 4px;
}

.datagrid-toolbar-badge {
	font-size: 10px;
	line-height: 1;
	padding: 1px 4px;
	border-radius: var(--app-radius-sm);
	background: var(--app-accent-strong);
	color: white;
	font-weight: 600;
}

.datagrid-toolbar-badge--dirty {
	background: var(--app-warning);
}

.datagrid-toolbar-filter-tag {
	font-size: 11px;
	line-height: 1;
	padding: 3px 8px;
	border: 1px solid color-mix(in srgb, var(--app-accent) 24%, var(--app-rule));
	border-radius: var(--app-radius-sm);
	background: var(--app-accent-soft);
	color: var(--app-accent-strong);
	font-weight: 500;
	white-space: nowrap;
}

@media (max-width: 760px) {
	.datagrid-toolbar {
		align-items: center;
	}

	.datagrid-toolbar-left,
	.datagrid-toolbar-right {
		flex-wrap: nowrap;
	}
}

@media (max-width: 1260px) {
	.datagrid-toolbar {
		scrollbar-width: none;
	}

	.datagrid-toolbar::-webkit-scrollbar {
		display: none;
	}

	.datagrid-toolbar-group-label {
		display: none;
	}

	.datagrid-toolbar-btn--with-label {
		min-width: 30px;
	}

	.datagrid-toolbar-btn--with-label span {
		display: none;
	}
}
</style>

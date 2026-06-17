<script setup lang="ts">
/**
 * DataGridHeaderMenu — DBeaver-style column header context menu.
 *
 * Shows on right-click (or click the dropdown indicator) on a column header.
 * Provides:
 * - Sort ASC / DESC / Clear sort
 * - Quick value filter presets (= value, != value, > value, etc.)
 * - Custom filter input
 * - Clear all filters / sorts for this column
 */
import { ArrowDownAZ, ArrowUpZA, Ban, Filter, FilterX, SearchX, X } from "lucide-vue-next"
import { nextTick, ref, watch } from "vue"
import { isNumericType } from "./filterParser"
import type { DataGridColumn, DataGridSort } from "./types"

const props = defineProps<{
	column: DataGridColumn
	/** Position to render the menu at (absolute). */
	x: number
	y: number
	/** Current sort state. */
	sort: DataGridSort | null
	/** Current filter value for this column. */
	filterValue: string
}>()

const emit = defineEmits<{
	(e: "sort", order: "asc" | "desc" | null): void
	(e: "filter", column: string, value: string): void
	(e: "clear-filter", column: string): void
	(e: "close"): void
}>()

const menuRef = ref<HTMLDivElement | null>(null)
const customFilterInput = ref("")
const showCustomInput = ref(false)
const customInputRef = ref<HTMLInputElement | null>(null)

const isCurrentSortColumn = ref(false)
const currentSortOrder = ref<"asc" | "desc" | null>(null)

watch(
	() => props.sort,
	(s) => {
		isCurrentSortColumn.value = s?.column === props.column.key
		currentSortOrder.value = isCurrentSortColumn.value ? (s?.order ?? null) : null
	},
	{ immediate: true }
)

watch(
	() => props.filterValue,
	(v) => {
		customFilterInput.value = v
	},
	{ immediate: true }
)

const isNumeric = ref(isNumericType(props.column.dataType))

function handleSort(order: "asc" | "desc" | null) {
	emit("sort", order)
	emit("close")
}

function applyQuickFilter(expr: string) {
	emit("filter", props.column.key, expr)
	emit("close")
}

function clearFilter() {
	emit("clear-filter", props.column.key)
	emit("close")
}

function openCustomFilter() {
	showCustomInput.value = true
	nextTick(() => {
		customInputRef.value?.focus()
	})
}

function submitCustomFilter() {
	const val = customFilterInput.value.trim()
	if (val) {
		emit("filter", props.column.key, val)
	}
	emit("close")
}

function handleCustomKeydown(event: KeyboardEvent) {
	if (event.key === "Enter") {
		event.preventDefault()
		submitCustomFilter()
	}
	if (event.key === "Escape") {
		event.preventDefault()
		emit("close")
	}
}

// Close on outside click
function handleBackdropClick(event: MouseEvent) {
	const target = event.target as HTMLElement
	if (menuRef.value && !menuRef.value.contains(target)) {
		emit("close")
	}
}

// Quick filter presets based on type
function getPresets(): Array<{ label: string; value: string }> {
	if (isNumeric.value) {
		return [
			{ label: `${props.column.label} = ..`, value: "= " },
			{ label: `${props.column.label} <> ..`, value: "!= " },
			{ label: `${props.column.label} > ..`, value: "> " },
			{ label: `${props.column.label} < ..`, value: "< " },
			{ label: `${props.column.label} >= ..`, value: ">= " },
			{ label: `${props.column.label} <= ..`, value: "<= " },
		]
	}
	return [
		{ label: `${props.column.label} = ..`, value: "= " },
		{ label: `${props.column.label} <> ..`, value: "!= " },
		{ label: `${props.column.label} LIKE ..`, value: "" },
	]
}
</script>

<template>
	<!-- Backdrop to capture outside clicks -->
	<Teleport to="body">
		<div class="header-menu-backdrop" @mousedown="handleBackdropClick">
			<div
				ref="menuRef"
				class="header-menu"
				:style="{ left: `${x}px`, top: `${y}px` }"
				@mousedown.stop
			>
				<!-- Sort section -->
				<button
					class="header-menu-item"
					:class="{ 'header-menu-item--active': currentSortOrder === 'asc' }"
					@click="handleSort('asc')"
				>
					<ArrowUpZA class="header-menu-icon" />
					<span>按 {{ column.label }} 升序</span>
				</button>
				<button
					class="header-menu-item"
					:class="{ 'header-menu-item--active': currentSortOrder === 'desc' }"
					@click="handleSort('desc')"
				>
					<ArrowDownAZ class="header-menu-icon" />
					<span>按 {{ column.label }} 降序</span>
				</button>
				<button
					v-if="isCurrentSortColumn"
					class="header-menu-item header-menu-item--danger"
					@click="handleSort(null)"
				>
					<X class="header-menu-icon" />
					<span>清除排序</span>
				</button>

				<div class="header-menu-sep" />

				<!-- Filter section header -->
				<div class="header-menu-label">
					<Filter class="header-menu-icon" />
					<span>按值过滤</span>
				</div>

				<!-- Quick filter presets -->
				<button
					v-for="preset in getPresets()"
					:key="preset.label"
					class="header-menu-item header-menu-item--indent"
					@click="openCustomFilter(); customFilterInput = preset.value"
				>
					<span class="font-mono text-xs">{{ preset.label }}</span>
				</button>

				<!-- NULL / NOT NULL -->
				<button
					class="header-menu-item header-menu-item--indent"
					@click="applyQuickFilter('NULL')"
				>
					<Ban class="header-menu-icon" />
					<span>IS NULL</span>
				</button>
				<button
					class="header-menu-item header-menu-item--indent"
					@click="applyQuickFilter('NOT NULL')"
				>
					<SearchX class="header-menu-icon" />
					<span>IS NOT NULL</span>
				</button>

				<div class="header-menu-sep" />

				<!-- Custom filter -->
				<div v-if="!showCustomInput">
					<button class="header-menu-item" @click="openCustomFilter">
						<Filter class="header-menu-icon" />
						<span>自定义过滤 …</span>
					</button>
				</div>
				<div v-else class="header-menu-custom">
					<input
						ref="customInputRef"
						v-model="customFilterInput"
						class="header-menu-input"
						:placeholder="isNumeric ? '例如: > 10 或 1..100' : '例如: test 或 = hello'"
						@keydown="handleCustomKeydown"
					/>
					<button class="header-menu-apply" @click="submitCustomFilter">
						应用
					</button>
				</div>

				<!-- Clear filter (if active) -->
				<template v-if="filterValue">
					<div class="header-menu-sep" />
					<button class="header-menu-item header-menu-item--danger" @click="clearFilter">
						<FilterX class="header-menu-icon" />
						<span>清除此列过滤</span>
					</button>
				</template>
			</div>
		</div>
	</Teleport>
</template>

<style scoped>
.header-menu-backdrop {
	position: fixed;
	inset: 0;
	z-index: 1000;
}

.header-menu {
	position: fixed;
	min-width: 220px;
	max-width: 320px;
	background: var(--app-surface-elevated);
	border: 1px solid var(--app-rule);
	border-radius: 8px;
	box-shadow: var(--app-shadow-whisper);
	padding: 4px 0;
	font-size: 13px;
	z-index: 1001;
}

.header-menu-item {
	display: flex;
	align-items: center;
	gap: 8px;
	width: 100%;
	padding: 6px 12px;
	border: none;
	background: transparent;
	color: var(--app-ink);
	cursor: pointer;
	text-align: left;
	font-size: 13px;
	line-height: 1.4;
	white-space: nowrap;
}

.header-menu-item:hover {
	background: var(--app-control-hover);
}

.header-menu-item--active {
	background: var(--app-accent-soft);
	color: var(--app-accent);
}

.header-menu-item--active:hover {
	background: var(--app-accent-soft);
}

.header-menu-item--indent {
	padding-left: 28px;
}

.header-menu-item--danger {
	color: var(--app-danger);
}

.header-menu-item--danger:hover {
	background: var(--app-danger-soft);
}

.header-menu-label {
	display: flex;
	align-items: center;
	gap: 8px;
	padding: 6px 12px 2px;
	color: var(--app-subtle);
	font-size: 11px;
	font-weight: 600;
	text-transform: uppercase;
	letter-spacing: 0.05em;
}

.header-menu-icon {
	width: 14px;
	height: 14px;
	flex-shrink: 0;
}

.header-menu-sep {
	height: 1px;
	background: var(--app-rule);
	margin: 4px 0;
}

.header-menu-custom {
	padding: 6px 12px;
	display: flex;
	gap: 6px;
}

.header-menu-input {
	flex: 1;
	padding: 4px 8px;
	border: 1px solid var(--app-rule-strong);
	border-radius: 4px;
	font-size: 12px;
	line-height: 20px;
	outline: none;
	background: var(--app-control);
	color: var(--app-ink);
}

.header-menu-input:focus {
	border-color: var(--app-accent);
	box-shadow: 0 0 0 2px color-mix(in srgb, var(--app-focus) 20%, transparent);
}

.header-menu-apply {
	padding: 4px 10px;
	border: none;
	border-radius: 4px;
	background: var(--app-accent);
	color: white;
	font-size: 12px;
	font-weight: 500;
	cursor: pointer;
	white-space: nowrap;
}

.header-menu-apply:hover {
	background: var(--app-accent-strong);
}
</style>

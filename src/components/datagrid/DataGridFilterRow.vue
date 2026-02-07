<script setup lang="ts">
/**
 * DataGridFilterRow — renders the header filter input row.
 * Each column gets a small input box. Pressing Enter commits and emits.
 */
import { X } from "lucide-vue-next"
import type { DataGridColumn } from "./types"

defineProps<{
	columns: DataGridColumn[]
	filters: Record<string, string>
	disabled: boolean
}>()

const emit = defineEmits<{
	(e: "update-filter", column: string, value: string): void
	(e: "clear-filter", column: string): void
	(e: "commit"): void
}>()

function onInput(column: string, event: Event) {
	const target = event.target as HTMLInputElement
	emit("update-filter", column, target.value)
}

function onKeydown(event: KeyboardEvent) {
	if (event.key === "Enter") {
		event.preventDefault()
		emit("commit")
	}
}

function clearFilter(column: string) {
	emit("clear-filter", column)
	emit("commit")
}
</script>

<template>
	<tr class="datagrid-filter-row">
		<td
			v-for="col in columns"
			:key="col.key"
			class="datagrid-filter-cell"
		>
			<div class="datagrid-filter-wrapper">
				<input
					type="text"
					class="datagrid-filter-input"
					:placeholder="`筛选…`"
					:value="filters[col.key] ?? ''"
					:disabled="disabled"
					@input="onInput(col.key, $event)"
					@keydown="onKeydown"
				/>
				<button
					v-if="filters[col.key]?.trim()"
					class="datagrid-filter-clear"
					title="清除筛选"
					@click="clearFilter(col.key)"
				>
					<X class="h-3 w-3" />
				</button>
			</div>
		</td>
	</tr>
</template>

<style scoped>
.datagrid-filter-row {
	background: #f1f5f9;
}

.datagrid-filter-cell {
	padding: 4px 6px;
	border-bottom: 2px solid #e2e8f0;
}

.datagrid-filter-wrapper {
	position: relative;
	display: flex;
	align-items: center;
}

.datagrid-filter-input {
	width: 100%;
	padding: 3px 22px 3px 8px;
	border: 1px solid #cbd5e1;
	border-radius: 4px;
	font-size: 12px;
	background: white;
	line-height: 18px;
}

.datagrid-filter-input:focus {
	border-color: #38bdf8;
	outline: none;
	box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.15);
}

.datagrid-filter-input:disabled {
	background: #f1f5f9;
	cursor: not-allowed;
}

.datagrid-filter-clear {
	position: absolute;
	right: 4px;
	display: flex;
	align-items: center;
	justify-content: center;
	width: 16px;
	height: 16px;
	border: none;
	border-radius: 50%;
	background: transparent;
	color: #94a3b8;
	cursor: pointer;
	padding: 0;
}

.datagrid-filter-clear:hover {
	color: #475569;
	background: #e2e8f0;
}
</style>

<script setup lang="ts">
/**
 * DataGridCell — renders a single table cell in view or edit mode.
 *
 * In view mode, it displays formatted content (null styling, vector preview, etc.).
 * In edit mode, it shows an input appropriate for the data type.
 */
import { computed, nextTick, ref, watch } from "vue"
import { formatCellValue } from "../../lib/formatters"
import { isBinaryType, isNumericType } from "./filterParser"

const props = defineProps<{
	value: unknown
	columnKey: string
	dataType: string
	nullable: boolean
	editable: boolean
	editing: boolean
	dirty: boolean
}>()

const emit = defineEmits<{
	(e: "commit", value: unknown): void
	(e: "cancel"): void
	(e: "start-edit"): void
	(e: "navigate", direction: "right" | "left" | "down" | "up"): void
}>()

const inputRef = ref<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | null>(null)
const editValue = ref("")

const isBinary = computed(() => isBinaryType(props.dataType))
const isNumeric = computed(() => isNumericType(props.dataType))
const isBoolType = computed(() => props.dataType.toLowerCase() === "boolean")
const isNull = computed(() => props.value === null || props.value === undefined)

// Initialize edit value when editing starts
watch(
	() => props.editing,
	(nowEditing) => {
		if (nowEditing) {
			if (props.value === null || props.value === undefined) {
				editValue.value = ""
			} else if (isBoolType.value) {
				editValue.value = String(props.value)
			} else if (Array.isArray(props.value)) {
				editValue.value = JSON.stringify(props.value)
			} else {
				editValue.value = String(props.value)
			}
			nextTick(() => {
				inputRef.value?.focus()
				if (inputRef.value instanceof HTMLInputElement) {
					inputRef.value.select()
				}
			})
		}
	}
)

function commitEdit() {
	const raw = editValue.value.trim()

	// NULL handling
	if (raw === "" || raw.toUpperCase() === "NULL") {
		if (props.nullable) {
			emit("commit", null)
		} else {
			emit("cancel")
		}
		return
	}

	if (isBoolType.value) {
		emit("commit", raw.toLowerCase() === "true")
		return
	}

	if (isNumeric.value) {
		const num = Number(raw)
		if (!Number.isNaN(num)) {
			emit("commit", num)
		} else {
			emit("cancel")
		}
		return
	}

	emit("commit", raw)
}

function handleKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case "Enter":
			event.preventDefault()
			commitEdit()
			nextTick(() => emit("navigate", "down"))
			break
		case "Tab":
			event.preventDefault()
			commitEdit()
			nextTick(() => emit("navigate", event.shiftKey ? "left" : "right"))
			break
		case "Escape":
			event.preventDefault()
			emit("cancel")
			break
	}
}

function handleDblClick() {
	if (props.editable && !props.editing) {
		emit("start-edit")
	}
}
</script>

<template>
	<td
		class="datagrid-cell"
		:class="{
			'datagrid-cell--editing': editing,
			'datagrid-cell--dirty': dirty && !editing,
			'datagrid-cell--null': isNull && !editing,
			'datagrid-cell--readonly': !editable,
		}"
		@dblclick="handleDblClick"
	>
		<!-- Edit mode -->
		<template v-if="editing">
			<select
				v-if="isBoolType"
				ref="inputRef"
				v-model="editValue"
				class="datagrid-cell-input"
				@keydown="handleKeydown"
				@blur="commitEdit"
			>
				<option value="true">true</option>
				<option value="false">false</option>
				<option v-if="nullable" value="">NULL</option>
			</select>
			<input
				v-else-if="isNumeric"
				ref="inputRef"
				v-model="editValue"
				type="text"
				inputmode="numeric"
				class="datagrid-cell-input"
				@keydown="handleKeydown"
				@blur="commitEdit"
			/>
			<input
				v-else
				ref="inputRef"
				v-model="editValue"
				type="text"
				class="datagrid-cell-input"
				@keydown="handleKeydown"
				@blur="commitEdit"
			/>
		</template>

		<!-- View mode -->
		<template v-else>
			<span v-if="isNull" class="datagrid-null">NULL</span>
			<span v-else-if="typeof value === 'boolean'" class="select-none">
				{{ value ? "✅" : "❌" }}
			</span>
			<span v-else-if="Array.isArray(value)" class="font-mono text-xs">
				<span class="mr-1 text-slate-400">[{{ value.length }}d]</span>
				{{ value.slice(0, 4).map((v) => formatCellValue(v)).join(", ")
				}}{{ value.length > 4 ? ", …" : "" }}
			</span>
			<span v-else-if="isBinary" class="font-mono text-xs text-slate-400">
				Binary
			</span>
			<span v-else>{{ formatCellValue(value) }}</span>
		</template>
	</td>
</template>

<style scoped>
.datagrid-cell {
	padding: 4px 12px;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
	max-width: 300px;
	cursor: default;
	position: relative;
	height: 32px;
	line-height: 24px;
}

.datagrid-cell--readonly {
	cursor: default;
}

.datagrid-cell--null .datagrid-null {
	color: #94a3b8;
	font-style: italic;
	user-select: none;
}

.datagrid-cell--dirty {
	background: #fffbeb;
}

.datagrid-cell--editing {
	padding: 2px 4px;
	background: #fefce8;
}

.datagrid-cell-input {
	width: 100%;
	padding: 2px 6px;
	border: 2px solid #38bdf8;
	border-radius: 3px;
	font-size: 13px;
	font-family: inherit;
	line-height: 20px;
	outline: none;
	background: white;
}

.datagrid-cell-input:focus {
	box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.2);
}
</style>

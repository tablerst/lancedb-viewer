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
const formattedValue = computed(() => formatCellValue(props.value))
const detailText = computed(() => formatInspectableValue(props.value))
const hasInspectableValue = computed(() => {
	if (isNull.value || isBinary.value || typeof props.value === "boolean") {
		return false
	}
	if (Array.isArray(props.value)) {
		return props.value.length > 4
	}
	if (typeof props.value === "object") {
		return props.value !== null
	}
	return formattedValue.value.length > 48
})
const previewText = computed(() => {
	if (Array.isArray(props.value)) {
		const preview = props.value
			.slice(0, 4)
			.map((v) => formatCellValue(v))
			.join(", ")
		return `[${props.value.length}d] ${preview}${props.value.length > 4 ? ", ..." : ""}`
	}
	return formattedValue.value
})
const previewBody = computed(() => {
	if (Array.isArray(props.value)) {
		return `${props.value
			.slice(0, 4)
			.map((v) => formatCellValue(v))
			.join(", ")}${props.value.length > 4 ? ", ..." : ""}`
	}
	return previewText.value
})

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

function formatInspectableValue(value: unknown): string {
	if (typeof value === "string") {
		return value
	}
	if (Array.isArray(value) || (value !== null && typeof value === "object")) {
		try {
			return JSON.stringify(value, null, 2)
		} catch {
			return String(value)
		}
	}
	return formatCellValue(value)
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
			<span
				v-else-if="typeof value === 'boolean'"
				class="datagrid-bool"
				:class="value ? 'datagrid-bool--true' : 'datagrid-bool--false'"
			>
				{{ value ? "true" : "false" }}
			</span>
			<NPopover
				v-else-if="hasInspectableValue"
				trigger="click"
				placement="bottom-start"
				:show-arrow="false"
				scrollable
				style="max-width: min(720px, calc(100vw - 48px)); max-height: 420px"
			>
				<template #trigger>
					<button
						class="datagrid-cell-detail"
						:title="previewText"
						@click.stop
						@dblclick.stop="emit('start-edit')"
					>
						<span v-if="Array.isArray(value)" class="datagrid-vector-dim">
							[{{ value.length }}d]
						</span>
						<span>{{ previewBody }}</span>
					</button>
				</template>
				<pre class="datagrid-cell-popover">{{ detailText }}</pre>
			</NPopover>
			<span v-else-if="Array.isArray(value)" class="datagrid-cell-mono">
				<span class="datagrid-vector-dim">[{{ value.length }}d]</span>
				{{ value.slice(0, 4).map((v) => formatCellValue(v)).join(", ") }}
			</span>
			<span v-else-if="isBinary" class="datagrid-cell-mono datagrid-cell-muted">
				Binary
			</span>
			<span v-else>{{ formattedValue }}</span>
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
	color: var(--app-ink);
}

.datagrid-cell--readonly {
	cursor: default;
}

.datagrid-cell--null .datagrid-null {
	color: var(--app-subtle);
	font-style: italic;
	user-select: none;
}

.datagrid-cell--dirty {
	background: var(--app-warning-soft);
}

.datagrid-cell--editing {
	padding: 2px 4px;
	background: var(--app-warning-soft);
}

.datagrid-cell-input {
	width: 100%;
	padding: 2px 6px;
	border: 2px solid var(--app-accent);
	border-radius: 3px;
	font-size: 13px;
	font-family: inherit;
	line-height: 20px;
	outline: none;
	background: var(--app-control);
	color: var(--app-ink);
}

.datagrid-cell-input:focus {
	box-shadow: 0 0 0 2px color-mix(in srgb, var(--app-focus) 22%, transparent);
}

.datagrid-cell-detail {
	display: inline-flex;
	max-width: 100%;
	align-items: center;
	gap: 4px;
	border: 0;
	background: transparent;
	color: inherit;
	cursor: zoom-in;
	font: inherit;
	padding: 0;
	text-align: left;
	vertical-align: top;
}

.datagrid-cell-detail span:last-child {
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.datagrid-cell-detail:hover {
	color: var(--app-accent);
	text-decoration: underline;
	text-underline-offset: 2px;
}

.datagrid-cell-mono,
.datagrid-cell-popover {
	font-family: var(--app-mono-font);
	font-size: 12px;
}

.datagrid-cell-popover {
	margin: 0;
	white-space: pre-wrap;
	word-break: break-word;
	color: var(--app-ink);
}

.datagrid-cell-muted,
.datagrid-vector-dim {
	color: var(--app-subtle);
}

.datagrid-bool {
	display: inline-flex;
	align-items: center;
	border-radius: 999px;
	padding: 1px 7px;
	font-family: var(--app-mono-font);
	font-size: 11px;
	font-weight: 600;
	line-height: 18px;
}

.datagrid-bool--true {
	background: var(--app-success-soft);
	color: var(--app-success);
}

.datagrid-bool--false {
	background: var(--app-danger-soft);
	color: var(--app-danger);
}
</style>

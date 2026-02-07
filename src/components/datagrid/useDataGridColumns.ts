import { computed, type Ref } from "vue"
import type { SchemaDefinition } from "../../ipc/v1"
import { isEditableType } from "./filterParser"
import type { DataGridColumn } from "./types"

/**
 * Generates DataGrid column definitions from a SchemaDefinition.
 */
export function useDataGridColumns(schema: Ref<SchemaDefinition | null>) {
	const columns = computed<DataGridColumn[]>(() => {
		if (!schema.value) return []
		return schema.value.fields.map((field) => ({
			key: field.name,
			label: field.name,
			dataType: field.dataType,
			nullable: field.nullable,
			editable: isEditableType(field.dataType),
		}))
	})

	const columnKeys = computed(() => columns.value.map((c) => c.key))

	return { columns, columnKeys }
}

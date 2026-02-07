<script setup lang="ts">
import type { DataTableColumns } from "naive-ui"
import { normalizeRow } from "../lib/formatters"

const props = withDefaults(
	defineProps<{
		columns: DataTableColumns<Record<string, unknown>>
		data: unknown[]
		loading?: boolean
		offset?: number
		error?: string
	}>(),
	{
		loading: false,
		offset: 0,
		error: "",
	}
)

const tableData = computed(() =>
	props.data.map((row, index) => ({
		__rowId: `${props.offset + index}`,
		...normalizeRow(row),
	}))
)
</script>

<template>
	<div>
		<NAlert v-if="error" type="error" :bordered="false" class="mb-2">
			{{ error }}
		</NAlert>
		<div v-if="loading && !tableData.length" class="space-y-2 py-4">
			<NSkeleton text :repeat="8" class="w-full" />
		</div>
		<NDataTable
			v-else
			class="data-result-table"
			size="small"
			:columns="columns"
			:data="tableData"
			:loading="loading"
			:bordered="false"
			:row-key="(row: Record<string, unknown>) => row.__rowId as string"
		/>
	</div>
</template>

<style scoped>
.data-result-table :deep(.n-data-table-th),
.data-result-table :deep(.n-data-table-td) {
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.data-result-table :deep(.table-header-ellipsis) {
	display: inline-block;
	max-width: 100%;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	vertical-align: bottom;
}
</style>

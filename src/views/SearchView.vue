<script setup lang="ts">
import { computed } from "vue";

import { useWorkspace } from "../composables/workspaceContext";

const { activeTableName } = useWorkspace();
const tableHint = computed(() => activeTableName.value ?? "尚未选择表");
</script>

<template>
	<NCard size="small" title="检索工作台" class="shadow-sm">
		<div class="space-y-3">
			<NEmpty description="检索功能正在建设中" />
			<p class="text-xs text-slate-500">
				当前表：{{ tableHint }}。请先在“资源浏览”中选择表，后续会在这里提供向量检索、
				全文检索与过滤查询的组合面板。
			</p>
		</div>
	</NCard>
</template>

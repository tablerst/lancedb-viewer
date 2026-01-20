<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import gsap from "gsap";
import { ChevronDown, ChevronRight, Plug, RefreshCcw, Table } from "lucide-vue-next";

import type { ConnectionState } from "../../composables/useConnection";
import type { StoredProfile } from "../../models/profile";
import { getConnectionKind, getConnectionKindLabel, getConnectionKindTagType } from "../../lib/connectionKind";

const props = defineProps<{
	profile: StoredProfile;
	state: ConnectionState;
	selected: boolean;
	collapsed: boolean;
}>();

const emit = defineEmits<{
	(e: "select"): void;
	(e: "connect"): void;
	(e: "refresh"): void;
	(e: "open-table", name: string): void;
}>();

const isConnected = computed(() => Boolean(props.state.connectionId.value));
const isConnecting = computed(() => props.state.isConnecting.value);
const isRefreshing = computed(() => props.state.isRefreshing.value);
const tables = computed(() => props.state.tables.value);
const activeTableName = computed(() => props.state.activeTableName.value);
const kind = computed(() => getConnectionKind(props.profile.uri));
const kindLabel = computed(() => getConnectionKindLabel(kind.value));
const tagType = computed(() => getConnectionKindTagType(kind.value));

const isExpanded = ref(true);
const tableListRef = ref<HTMLDivElement | null>(null);
const tableListHeight = computed(() => Math.min(tables.value.length * 32, 200));

watch(
	() => props.collapsed,
	(collapsed) => {
		if (collapsed) {
			isExpanded.value = false;
		}
	},
);

watch([isExpanded, tableListHeight, isConnected], async ([expanded, height, connected]) => {
	await nextTick();
	const el = tableListRef.value;
	if (!el) {
		return;
	}
	const targetHeight = expanded && connected ? height : 0;
	gsap.to(el, { height: targetHeight, duration: 0.2, ease: "power2.out" });
});

function toggleExpanded() {
	if (!isConnected.value) {
		return;
	}
	isExpanded.value = !isExpanded.value;
}
</script>

<template>
	<NCard
		size="small"
		class="shadow-sm"
		:class="[
			selected ? 'border-sky-200 ring-1 ring-sky-200' : '',
			collapsed ? 'min-h-[92px]' : '',
		]"
	>
		<div class="flex items-start justify-between gap-2">
			<button class="flex flex-1 items-start gap-2 text-left" @click="emit('select')">
				<span
					class="mt-1 h-2 w-2 rounded-full"
					:class="isConnected ? 'bg-emerald-500' : 'bg-slate-300'"
				/>
				<div v-if="!collapsed" class="min-w-0">
					<div class="truncate text-sm font-semibold text-slate-800">
						{{ profile.name }}
					</div>
					<div class="truncate text-xs text-slate-500">{{ profile.uri }}</div>
				</div>
			</button>
			<NTag size="small" :type="tagType">{{ kindLabel }}</NTag>
		</div>

		<div v-if="!collapsed" class="mt-2 flex flex-wrap items-center gap-2">
			<NButton
				size="tiny"
				type="primary"
				:loading="isConnecting"
				@click.stop="emit('connect')"
			>
				<Plug class="h-3 w-3" />
				<span class="ml-1">连接</span>
			</NButton>
			<NButton
				size="tiny"
				quaternary
				:disabled="!isConnected || isRefreshing"
				@click.stop="emit('refresh')"
			>
				<RefreshCcw class="h-3 w-3" />
				<span class="ml-1">刷新</span>
			</NButton>
			<NButton size="tiny" quaternary :disabled="!isConnected" @click.stop="toggleExpanded">
				<ChevronDown v-if="isExpanded" class="h-3 w-3" />
				<ChevronRight v-else class="h-3 w-3" />
				<span class="ml-1">表</span>
			</NButton>
		</div>

		<div v-if="!collapsed" class="mt-3">
			<div class="flex items-center justify-between text-xs text-slate-500">
				<span>表列表</span>
				<span v-if="!isConnected">未连接</span>
				<span v-else>共 {{ tables.length }} 张</span>
			</div>
			<div ref="tableListRef" class="mt-2 overflow-hidden" style="height: 0;">
				<NVirtualList
					v-if="isConnected"
					:items="tables"
					:item-size="32"
					:style="{ height: `${tableListHeight}px` }"
				>
					<template #default="{ item }">
						<NButton
							text
							class="w-full justify-start"
							:type="item.name === activeTableName ? 'primary' : 'default'"
							@click.stop="emit('open-table', item.name)"
						>
							<Table class="h-3 w-3" />
							<span class="ml-2 truncate">{{ item.name }}</span>
						</NButton>
					</template>
				</NVirtualList>
			</div>
		</div>
	</NCard>
</template>

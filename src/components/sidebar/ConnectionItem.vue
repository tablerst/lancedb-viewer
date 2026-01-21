<script setup lang="ts">
import { ChevronDown, ChevronRight, Pencil, Plug, RefreshCcw, Table, Trash2 } from "lucide-vue-next"
import { computed, ref, watch } from "vue"

import type { ConnectionState } from "../../composables/useConnection"
import { formatTimestamp } from "../../lib/formatters"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "../../lib/connectionKind"
import type { StoredProfile } from "../../models/profile"

const props = defineProps<{
	profile: StoredProfile
	state: ConnectionState | null
	selected: boolean
	collapsed: boolean
}>()

const emit = defineEmits<{
	(e: "select"): void
	(e: "connect"): void
	(e: "refresh"): void
	(e: "open-table", name: string): void
	(e: "edit"): void
	(e: "delete"): void
}>()

const isConnected = computed(() => Boolean(props.state?.connectionId?.value))
const isConnecting = computed(() => props.state?.isConnecting?.value ?? false)
const isRefreshing = computed(() => props.state?.isRefreshing?.value ?? false)
const tables = computed(() => props.state?.tables?.value ?? [])
const activeTableName = computed(() => props.state?.activeTableName?.value ?? null)
const kind = computed(() => getConnectionKind(props.profile.uri))
const kindLabel = computed(() => getConnectionKindLabel(kind.value))
const tagType = computed(() => getConnectionKindTagType(kind.value))
const statusDotClass = computed(() => {
	if (isConnecting.value) {
		return "bg-amber-400 animate-pulse"
	}
	if (isConnected.value) {
		return "bg-emerald-500"
	}
	return "bg-slate-300"
})
const statusText = computed(() => {
	if (isConnecting.value) {
		return "连接中"
	}
	if (isConnected.value) {
		return "已连接"
	}
	return "未连接"
})
const statusTextClass = computed(() => {
	if (isConnecting.value) {
		return "text-amber-600"
	}
	if (isConnected.value) {
		return "text-emerald-600"
	}
	return "text-slate-500"
})
const showCollapsedStatus = computed(() => isConnecting.value || !isConnected.value)
const lastConnectedLabel = computed(() => formatTimestamp(props.profile.lastConnectedAt))
const connectLabel = computed(() => (isConnected.value ? "重连" : "连接"))
const collapsedTitle = computed(() => {
	if (isConnecting.value || !isConnected.value) {
		return `${props.profile.name} · ${statusText.value} · ${kindLabel.value}`
	}
	return `${props.profile.name} · ${kindLabel.value}`
})

const isExpanded = ref(true)
const tableListHeight = computed(() => Math.min(tables.value.length * 32, 200))
const tableListContainerHeight = computed(() => {
	if (!isConnected.value) {
		return 0
	}
	if (!isExpanded.value) {
		return 0
	}
	return tableListHeight.value
})

watch(
	() => props.collapsed,
	(collapsed) => {
		if (collapsed) {
			isExpanded.value = false
		}
	}
)

function toggleExpanded() {
	if (!isConnected.value) {
		return
	}
	isExpanded.value = !isExpanded.value
}
</script>

<template>
	<NCard
		size="small"
		class="shadow-sm transition-shadow hover:shadow-md"
		:class="[
			selected ? 'border-sky-200 ring-1 ring-sky-200' : '',
			collapsed ? 'min-h-[104px] bg-slate-50/40 hover:bg-slate-50/70' : '',
		]"
	>
		<div v-if="collapsed" class="flex flex-col items-center gap-2">
			<button
				class="group flex w-full flex-col items-center gap-1.5 rounded-md px-2 py-1 text-center transition hover:bg-slate-100/70 focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-400"
				:title="collapsedTitle"
				@click="emit('select')"
			>
				<div class="flex items-center gap-1.5">
					<span class="h-2 w-2 rounded-full" :class="statusDotClass" />
					<div
						class="max-w-full truncate text-[11px] font-semibold leading-tight text-slate-800"
						:title="profile.name"
					>
						{{ profile.name }}
					</div>
				</div>
				<div v-if="showCollapsedStatus" class="text-[10px] font-medium" :class="statusTextClass">
					{{ statusText }}
				</div>
			</button>
			<div class="flex items-center justify-center">
				<NTag size="small" :type="tagType" :bordered="false" class="text-[10px]">
					{{ kindLabel }}
				</NTag>
			</div>
		</div>

		<div v-else>
			<div class="flex items-start justify-between gap-3">
				<button class="flex min-w-0 flex-1 text-left" @click="emit('select')">
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="h-2 w-2 rounded-full" :class="statusDotClass" />
							<div
								class="truncate text-sm font-semibold text-slate-800"
								:title="profile.name"
							>
								{{ profile.name }}
							</div>
						</div>
						<div class="truncate text-xs text-slate-500" :title="profile.uri">
							{{ profile.uri }}
						</div>
					</div>
				</button>
				<div class="flex shrink-0 items-start">
					<NTag size="small" :type="tagType">{{ kindLabel }}</NTag>
				</div>
			</div>

			<div class="mt-1 text-[11px] text-slate-500">最近连接：{{ lastConnectedLabel }}</div>

			<div class="mt-2 flex flex-wrap items-center justify-between gap-2">
				<div class="flex flex-wrap items-center gap-2">
					<NButton
						size="tiny"
						type="primary"
						:loading="isConnecting"
						:disabled="isConnecting"
						@click.stop="emit('connect')"
					>
						<Plug class="h-3 w-3" />
						<span class="ml-1">{{ connectLabel }}</span>
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
				</div>
				<div class="flex items-center gap-1">
					<NButton size="tiny" quaternary @click.stop="emit('edit')">
						<Pencil class="h-3 w-3" />
						<span class="ml-1">编辑</span>
					</NButton>
					<NPopconfirm
						positive-text="删除"
						negative-text="取消"
						@positive-click="emit('delete')"
					>
						<template #trigger>
							<NButton size="tiny" quaternary type="error" @click.stop="() => {}">
								<Trash2 class="h-3 w-3" />
								<span class="ml-1">删除</span>
							</NButton>
						</template>
						确定删除该连接档案吗？该操作不可撤销。
					</NPopconfirm>
				</div>
			</div>

			<div class="mt-3">
				<div class="flex items-center justify-between text-xs text-slate-500">
					<NButton
						size="tiny"
						quaternary
						:disabled="!isConnected"
						@click.stop="toggleExpanded"
					>
						<ChevronDown v-if="isExpanded" class="h-3 w-3" />
						<ChevronRight v-else class="h-3 w-3" />
						<span class="ml-1">表列表</span>
					</NButton>
					<div class="flex items-center gap-2">
						<span v-if="!isConnected">未连接</span>
						<span v-else>共 {{ tables.length }} 张</span>
					</div>
				</div>
				<div
					class="mt-2 overflow-hidden rounded-lg bg-slate-50/70 transition-[height] duration-200 ease-out"
					:style="{ height: `${tableListContainerHeight}px` }"
				>
					<div v-if="isConnected" class="border-l border-slate-200/80 pl-2">
						<NVirtualList
							:items="tables"
							:item-size="32"
							:style="{ height: `${tableListHeight}px` }"
						>
							<template #default="{ item }">
								<NButton
									text
									class="w-full justify-start rounded-md px-3 py-1"
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
			</div>
		</div>
	</NCard>
</template>

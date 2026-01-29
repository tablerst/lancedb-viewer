<script setup lang="ts">
import { ChevronDown, ChevronRight, MoreHorizontal, Table } from "lucide-vue-next"
import { computed, ref, watch } from "vue"

import type { ConnectionState } from "../../composables/useConnection"
import {
	getConnectionKind,
	getConnectionKindLabel,
	getConnectionKindTagType,
} from "../../lib/connectionKind"
import { formatTimestamp } from "../../lib/formatters"
import type { StoredProfile } from "../../models/profile"

const props = defineProps<{
	profile: StoredProfile
	state: ConnectionState | null
	selected: boolean
	collapsed: boolean
}>()

const emit = defineEmits<{
	(e: "select"): void
	(e: "open-table", name: string): void
	(e: "open-menu", event: MouseEvent): void
}>()

const isConnected = computed(() => Boolean(props.state?.connectionId?.value))
const isConnecting = computed(() => props.state?.isConnecting?.value ?? false)
const isDisconnecting = computed(() => props.state?.isDisconnecting?.value ?? false)
const tables = computed(() => props.state?.tables?.value ?? [])
const activeTableName = computed(() => props.state?.activeTableName?.value ?? null)
const kind = computed(() => getConnectionKind(props.profile.uri))
const kindLabel = computed(() => getConnectionKindLabel(kind.value))
const tagType = computed(() => getConnectionKindTagType(kind.value))
const statusDotClass = computed(() => {
	if (isConnecting.value) {
		return "bg-amber-400 animate-pulse"
	}
	if (isDisconnecting.value) {
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
	if (isDisconnecting.value) {
		return "断开中"
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
	if (isDisconnecting.value) {
		return "text-amber-600"
	}
	if (isConnected.value) {
		return "text-emerald-600"
	}
	return "text-slate-500"
})
const showCollapsedStatus = computed(() => isConnecting.value || isDisconnecting.value)
const lastConnectedLabel = computed(() => formatTimestamp(props.profile.lastConnectedAt))
const collapsedTitle = computed(() => {
	return `${props.profile.name} · ${statusText.value} · ${kindLabel.value}`
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

function openMenu(event: MouseEvent) {
	emit("open-menu", event)
}

function handleContextMenu(event: MouseEvent) {
	event.preventDefault()
	openMenu(event)
}
</script>

<template>
	<NCard
		size="small"
		class="group shadow-sm transition-shadow hover:shadow-md"
		:class="[
			selected ? 'border-sky-200 ring-1 ring-sky-200' : '',
			collapsed ? 'min-h-[84px] bg-slate-50/40 hover:bg-slate-50/70' : '',
		]"
		:content-style="collapsed ? { padding: '10px 8px' } : undefined"
		@contextmenu="handleContextMenu"
	>
		<div v-if="collapsed" class="flex flex-col items-center gap-1.5">
			<button
				class="group flex w-full flex-col items-center gap-1 rounded-md px-2 py-1.5 text-center transition hover:bg-slate-100/70 focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-400"
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
				<div
					v-if="showCollapsedStatus"
					class="text-[10px] font-medium leading-tight"
					:class="statusTextClass"
				>
					{{ statusText }}
				</div>
			</button>
			<NTag size="small" :type="tagType" :bordered="false" class="text-[10px]">
				{{ kindLabel }}
			</NTag>
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
				<div class="flex shrink-0 items-center gap-1">
					<NButton
						size="tiny"
						quaternary
						circle
						class="opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100"
						aria-label="更多操作"
						title="更多操作"
						@click.stop="openMenu"
					>
						<MoreHorizontal class="h-4 w-4" />
					</NButton>
					<NTag size="small" :type="tagType">{{ kindLabel }}</NTag>
				</div>
			</div>

			<div class="mt-1 text-[11px] text-slate-500">最近连接：{{ lastConnectedLabel }}</div>

			<div class="mt-2 text-[11px] text-slate-400">
				右键该连接以查看更多操作
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

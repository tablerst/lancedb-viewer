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
	(e: "connect"): void
	(e: "disconnect"): void
	(e: "refresh"): void
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
const lastConnectedLabel = computed(() => formatTimestamp(props.profile.lastConnectedAt))
const initials = computed(() => {
	const name = props.profile.name.trim()
	if (!name) return "?"
	const parts = name.split(/[\s_\-./\\]+/).filter(Boolean)
	if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
	return name.slice(0, 2).toUpperCase()
})
const avatarBgClass = computed(() => {
	if (isConnecting.value || isDisconnecting.value) return "bg-amber-100 text-amber-700"
	if (isConnected.value) return "bg-sky-100 text-sky-700"
	return "bg-slate-100 text-slate-500"
})
const tableCount = computed(() => tables.value.length)
const collapsedLabel = computed(() => {
	const tableSummary = isConnected.value ? `，${tableCount.value} 张表` : ""
	return `${props.profile.name}，${statusText.value}，${kindLabel.value}${tableSummary}`
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
		class="group transform-gpu will-change-transform transition-[transform,box-shadow,border-color,background-color,opacity] duration-200 ease-out focus-within:border-sky-200"
		:class="[
			selected
				? 'border-sky-200 shadow-[0_1px_2px_rgba(15,23,42,0.06),0_6px_18px_rgba(14,165,233,0.10)] hover:-translate-y-0.5 hover:border-sky-300 hover:shadow-[0_2px_6px_rgba(15,23,42,0.08),0_12px_26px_rgba(14,165,233,0.16)]'
				: 'hover:-translate-y-0.5 hover:border-slate-300 hover:shadow-md',
			collapsed
				? selected
					? 'bg-sky-50/40 hover:bg-sky-50/60'
					: 'bg-slate-50/40 hover:bg-slate-50/70'
				: '',
			!isConnected && !isConnecting && !isDisconnecting ? 'opacity-70 hover:opacity-100' : '',
		]"
		:content-style="collapsed ? { padding: '6px 4px' } : undefined"
		@contextmenu="handleContextMenu"
	>
		<!-- V-2: Connected left indicator -->
		<div
			v-if="isConnected && !collapsed"
			class="absolute left-0 top-0 h-full w-[3px] rounded-l-xl bg-sky-400 transition-opacity duration-200"
		/>
		<div
			v-else-if="(isConnecting || isDisconnecting) && !collapsed"
			class="absolute left-0 top-0 h-full w-[3px] animate-pulse rounded-l-xl bg-amber-400 transition-opacity duration-200"
		/>
		<!-- Collapsed left indicator (consistent with expanded) -->
		<div
			v-if="isConnected && collapsed"
			class="absolute left-0 top-0 h-full w-[3px] rounded-l-xl bg-sky-400 transition-opacity duration-200"
		/>
		<div
			v-else-if="(isConnecting || isDisconnecting) && collapsed"
			class="absolute left-0 top-0 h-full w-[3px] animate-pulse rounded-l-xl bg-amber-400 transition-opacity duration-200"
		/>
		<div v-if="collapsed" class="flex flex-col items-center gap-0.5">
			<NTooltip placement="right" :delay="300">
				<template #trigger>
					<button
						class="group/btn flex w-full flex-col items-center gap-1 rounded-md px-1 py-1 text-center transition hover:bg-slate-100/70 focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-400"
						:aria-label="collapsedLabel"
						:title="collapsedLabel"
						@click="emit('select')"
					>
						<div class="relative">
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold transition-colors"
								:class="avatarBgClass"
							>
								{{ initials }}
							</div>
							<span
								class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-white"
								:class="statusDotClass"
							/>
						</div>
						<span class="max-w-full truncate text-[10px] font-medium text-slate-500">
							{{ isConnected ? `${tableCount} 表` : kindLabel }}
						</span>
					</button>
				</template>
				<div class="space-y-1 text-xs">
					<div class="font-semibold">{{ profile.name }}</div>
					<div class="text-slate-400">{{ profile.uri }}</div>
					<div class="flex items-center gap-2">
						<span :class="statusTextClass">{{ statusText }}</span>
						<span class="text-slate-400">·</span>
						<span class="text-slate-400">{{ kindLabel }}</span>
					</div>
					<div v-if="isConnected" class="text-slate-400">
						{{ tableCount }} 张表
					</div>
				</div>
			</NTooltip>
		</div>

		<div v-else>
			<div class="flex items-start justify-between gap-3">
				<button
					class="flex min-w-0 flex-1 rounded-md text-left focus:outline-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-400"
					@click="emit('select')"
				>
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

			<!-- I-1: Inline action buttons -->
			<div class="mt-2 flex items-center gap-2">
				<template v-if="!isConnected && !isConnecting && !isDisconnecting">
					<NButton
						size="tiny"
						type="primary"
						@click.stop="emit('connect')"
					>
						连接
					</NButton>
				</template>
				<template v-else-if="isConnecting">
					<NButton size="tiny" type="primary" :loading="true" disabled>
						连接中
					</NButton>
				</template>
				<template v-else-if="isDisconnecting">
					<NButton size="tiny" quaternary :loading="true" disabled>
						断开中
					</NButton>
				</template>
				<template v-else>
					<NButton
						size="tiny"
						secondary
						@click.stop="emit('refresh')"
					>
						刷新
					</NButton>
					<NButton
						size="tiny"
						quaternary
						@click.stop="emit('disconnect')"
					>
						断开
					</NButton>
				</template>
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

<style scoped>
:deep(.n-card) {
	background: var(--app-surface-elevated);
	border-color: var(--app-rule);
	color: var(--app-ink);
}

:deep(.n-card:hover) {
	border-color: var(--app-rule-strong);
}

:deep(.n-card__content) {
	color: var(--app-ink);
}

.group {
	color: var(--app-ink);
}
</style>

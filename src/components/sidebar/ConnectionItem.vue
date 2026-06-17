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
		return "connection-status-dot--pending animate-pulse"
	}
	if (isDisconnecting.value) {
		return "connection-status-dot--pending animate-pulse"
	}
	if (isConnected.value) {
		return "connection-status-dot--connected"
	}
	return "connection-status-dot--idle"
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
		return "connection-status-text--pending"
	}
	if (isDisconnecting.value) {
		return "connection-status-text--pending"
	}
	if (isConnected.value) {
		return "connection-status-text--connected"
	}
	return "connection-status-text--idle"
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
	if (isConnecting.value || isDisconnecting.value) return "connection-avatar--pending"
	if (isConnected.value) return "connection-avatar--connected"
	return "connection-avatar--idle"
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
		class="group connection-card transition-colors duration-150 ease-out"
		:class="[
			selected ? 'connection-card--selected' : '',
			collapsed ? 'connection-card--collapsed' : '',
			!isConnected && !isConnecting && !isDisconnecting ? 'connection-card--idle' : '',
		]"
		:content-style="collapsed ? { padding: '6px 4px' } : undefined"
		@contextmenu="handleContextMenu"
	>
		<!-- V-2: Connected left indicator -->
		<div
			v-if="isConnected && !collapsed"
			class="connection-state-bar connection-state-bar--connected"
		/>
		<div
			v-else-if="(isConnecting || isDisconnecting) && !collapsed"
			class="connection-state-bar connection-state-bar--pending animate-pulse"
		/>
		<!-- Collapsed left indicator (consistent with expanded) -->
		<div
			v-if="isConnected && collapsed"
			class="connection-state-bar connection-state-bar--connected"
		/>
		<div
			v-else-if="(isConnecting || isDisconnecting) && collapsed"
			class="connection-state-bar connection-state-bar--pending animate-pulse"
		/>
		<div v-if="collapsed" class="flex flex-col items-center gap-0.5">
			<NTooltip placement="right" :delay="300">
				<template #trigger>
					<button
						class="connection-collapsed-trigger group/btn flex w-full flex-col items-center gap-1 px-1 py-1 text-center transition focus:outline-none"
						:aria-label="collapsedLabel"
						:title="collapsedLabel"
						@click="emit('select')"
					>
						<div class="relative">
							<div
								class="connection-avatar flex h-8 w-8 items-center justify-center text-xs font-bold transition-colors"
								:class="avatarBgClass"
							>
								{{ initials }}
							</div>
							<span
								class="connection-collapsed-dot absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2"
								:class="statusDotClass"
							/>
						</div>
						<span class="max-w-full truncate text-[10px] font-medium text-[var(--app-muted)]">
							{{ isConnected ? `${tableCount} 表` : kindLabel }}
						</span>
					</button>
				</template>
				<div class="space-y-1 text-xs">
					<div class="font-semibold">{{ profile.name }}</div>
					<div class="text-[var(--app-muted)]">{{ profile.uri }}</div>
					<div class="flex items-center gap-2">
						<span :class="statusTextClass">{{ statusText }}</span>
						<span class="text-[var(--app-subtle)]">·</span>
						<span class="text-[var(--app-muted)]">{{ kindLabel }}</span>
					</div>
					<div v-if="isConnected" class="text-[var(--app-muted)]">
						{{ tableCount }} 张表
					</div>
				</div>
			</NTooltip>
		</div>

		<div v-else>
			<div class="flex items-start justify-between gap-3">
				<button
					class="connection-main-trigger flex min-w-0 flex-1 rounded-md text-left focus:outline-none"
					@click="emit('select')"
				>
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="h-2 w-2 rounded-full" :class="statusDotClass" />
							<div
								class="truncate text-sm font-semibold text-[var(--app-ink-strong)]"
								:title="profile.name"
							>
								{{ profile.name }}
							</div>
						</div>
						<div class="truncate text-xs text-[var(--app-muted)]" :title="profile.uri">
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

			<div class="mt-1 text-[11px] text-[var(--app-muted)]">最近连接：{{ lastConnectedLabel }}</div>

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
				<div class="flex items-center justify-between text-xs text-[var(--app-muted)]">
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
					class="connection-table-list mt-2 overflow-hidden transition-[height] duration-150 ease-out"
					:style="{ height: `${tableListContainerHeight}px` }"
				>
					<div v-if="isConnected" class="border-l border-[var(--app-rule)] pl-2">
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
:deep(.connection-card.n-card) {
	position: relative;
	background: var(--app-surface-elevated);
	border-color: var(--app-rule);
	color: var(--app-ink);
	box-shadow: none;
}

:deep(.connection-card.n-card:hover) {
	border-color: var(--app-rule-strong);
	background: var(--app-control);
}

:deep(.connection-card.n-card.connection-card--selected) {
	border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-rule));
	background: var(--app-accent-soft);
}

:deep(.connection-card.n-card.connection-card--idle) {
	background: color-mix(in srgb, var(--app-surface-elevated) 72%, var(--app-surface-panel));
}

:deep(.connection-card.n-card.connection-card--collapsed) {
	text-align: center;
}

:deep(.n-card__content) {
	color: var(--app-ink);
}

.group {
	color: var(--app-ink);
}

.connection-state-bar {
	position: absolute;
	top: 0;
	left: 0;
	width: 2px;
	height: 100%;
	border-radius: var(--app-radius-lg) 0 0 var(--app-radius-lg);
	opacity: 0.85;
}

.connection-state-bar--connected {
	background: var(--app-success);
}

.connection-state-bar--pending {
	background: var(--app-warning);
}

.connection-collapsed-trigger,
.connection-main-trigger {
	border-radius: var(--app-radius-md);
	color: var(--app-ink);
}

.connection-collapsed-trigger:hover,
.connection-main-trigger:hover {
	background: color-mix(in srgb, var(--app-control-hover) 72%, transparent);
}

.connection-collapsed-trigger:focus-visible,
.connection-main-trigger:focus-visible {
	box-shadow: 0 0 0 2px color-mix(in srgb, var(--app-focus) 28%, transparent);
}

.connection-avatar {
	border: 1px solid var(--app-rule);
	border-radius: var(--app-radius-md);
}

.connection-avatar--connected {
	background: var(--app-accent-soft);
	color: var(--app-accent-strong);
}

.connection-avatar--pending {
	background: var(--app-warning-soft);
	color: var(--app-warning);
}

.connection-avatar--idle {
	background: var(--app-surface-panel-muted);
	color: var(--app-muted);
}

.connection-collapsed-dot {
	border-color: var(--app-surface-elevated);
}

.connection-status-dot--connected {
	background: var(--app-success);
}

.connection-status-dot--pending {
	background: var(--app-warning);
}

.connection-status-dot--idle {
	background: var(--app-subtle);
}

.connection-status-text--connected {
	color: var(--app-success);
}

.connection-status-text--pending {
	color: var(--app-warning);
}

.connection-status-text--idle {
	color: var(--app-muted);
}

.connection-table-list {
	border-radius: var(--app-radius-md);
	background: color-mix(in srgb, var(--app-surface-panel-muted) 58%, transparent);
}
</style>

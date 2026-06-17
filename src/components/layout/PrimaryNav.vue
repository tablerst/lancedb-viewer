<script setup lang="ts">
import { Database, Key, LayoutGrid, Moon, Search, Settings, Sun } from "lucide-vue-next"
import { computed } from "vue"
import { useRoute, useRouter } from "vue-router"

import { useTheme } from "../../composables/useTheme"
import { useWorkspace } from "../../composables/workspaceContext"

type NavKey = "resources" | "search" | "vault" | "capabilities"

type NavItem = {
	key: NavKey
	label: string
	icon: typeof LayoutGrid
	to: string
}

const route = useRoute()
const router = useRouter()

const { activeProfileId } = useWorkspace()
const { isDark, toggle: toggleTheme } = useTheme()

const mainItems = computed<NavItem[]>(() => {
	const activeId = activeProfileId.value
	return [
		{
			key: "resources",
			label: "资源",
			icon: LayoutGrid,
			to: activeId ? `/connections/${activeId}` : "/",
		},
		{
			key: "search",
			label: "检索",
			icon: Search,
			to: activeId ? `/connections/${activeId}/search` : "/search",
		},
		{
			key: "vault",
			label: "凭证库",
			icon: Key,
			to: "/vault/credentials",
		},
	]
})

const bottomItems = computed<NavItem[]>(() => [
	{
		key: "capabilities",
		label: "设置",
		icon: Settings,
		to: "/capabilities",
	},
])

function isActive(key: NavKey) {
	if (key === "resources") {
		return (
			route.path === "/" ||
			(route.path.startsWith("/connections") && !route.path.endsWith("/search"))
		)
	}
	if (key === "search") {
		return (
			route.path === "/search" ||
			(route.path.startsWith("/connections/") && route.path.endsWith("/search"))
		)
	}
	if (key === "vault") {
		return route.path === "/vault/credentials" || route.path === "/credentials"
	}
	return route.path === "/capabilities"
}

function navigate(to: string) {
	if (route.path === to) {
		return
	}
	void router.push(to)
}
</script>

<template>
	<aside
		class="primary-nav"
		aria-label="全局导航"
	>
		<div class="flex items-center justify-center px-1.5 py-3">
			<div
				class="primary-nav-brand"
				title="LanceDB Studio"
			>
				<Database class="h-3.5 w-3.5" />
			</div>
		</div>

		<nav class="flex flex-1 flex-col items-center gap-0.5 px-1 pt-1">
			<button
				v-for="item in mainItems"
				:key="item.key"
				class="primary-nav-button"
				:class="{ 'primary-nav-button--active': isActive(item.key) }"
				:aria-label="item.label"
				:aria-current="isActive(item.key) ? 'page' : undefined"
				:title="item.label"
				@click="navigate(item.to)"
			>
				<component :is="item.icon" class="h-4 w-4" />
				<span>{{ item.label }}</span>
			</button>
		</nav>

		<div class="flex flex-col items-center gap-0.5 px-1 pb-3">
			<button
				class="primary-nav-button"
				:title="isDark ? '切换到浅色模式' : '切换到深色模式'"
				:aria-label="isDark ? '切换到浅色模式' : '切换到深色模式'"
				@click="toggleTheme"
			>
				<Sun v-if="isDark" class="h-4 w-4" />
				<Moon v-else class="h-4 w-4" />
				<span>{{ isDark ? "浅色" : "深色" }}</span>
			</button>
			<button
				v-for="item in bottomItems"
				:key="item.key"
				class="primary-nav-button"
				:class="{ 'primary-nav-button--active': isActive(item.key) }"
				:aria-label="item.label"
				:aria-current="isActive(item.key) ? 'page' : undefined"
				:title="item.label"
				@click="navigate(item.to)"
			>
				<component :is="item.icon" class="h-4 w-4" />
				<span>{{ item.label }}</span>
			</button>
		</div>
	</aside>
</template>

<style scoped>
.primary-nav {
	display: flex;
	width: 58px;
	height: 100%;
	flex-shrink: 0;
	flex-direction: column;
	border-right: 1px solid rgb(148 163 184 / 0.18);
	background: var(--app-nav);
	color: var(--app-nav-muted);
}

.primary-nav-brand {
	display: flex;
	width: 32px;
	height: 32px;
	align-items: center;
	justify-content: center;
	border-radius: 8px;
	background: linear-gradient(135deg, #0ea5e9, #14b8a6);
	color: white;
	box-shadow: 0 8px 18px rgb(14 165 233 / 0.25);
}

.primary-nav-button {
	display: flex;
	width: 100%;
	min-height: 48px;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 3px;
	border-radius: 8px;
	padding: 6px 2px;
	color: var(--app-nav-muted);
	font-size: 11px;
	font-weight: 500;
	line-height: 1.05;
	transition: background-color 120ms ease, color 120ms ease, box-shadow 120ms ease;
}

.primary-nav-button:hover {
	background: rgb(255 255 255 / 0.08);
	color: white;
}

.primary-nav-button:focus-visible {
	outline: 2px solid var(--app-focus);
	outline-offset: 2px;
}

.primary-nav-button--active {
	background: rgb(14 165 233 / 0.18);
	color: #e0f7ff;
	box-shadow: inset 3px 0 0 #38bdf8;
}

.primary-nav-button span {
	max-width: 100%;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}
</style>

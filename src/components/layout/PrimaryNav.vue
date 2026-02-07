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
	<aside class="flex h-full w-12 shrink-0 flex-col border-r border-slate-200 bg-white">
		<div class="flex items-center justify-center px-1.5 py-3">
			<div
				class="flex h-7 w-7 items-center justify-center rounded-lg bg-sky-500 text-white"
				title="LanceDB Studio"
			>
				<Database class="h-3.5 w-3.5" />
			</div>
		</div>

		<nav class="flex flex-1 flex-col items-center gap-0.5 px-1 pt-1">
			<button
				v-for="item in mainItems"
				:key="item.key"
				class="flex w-full flex-col items-center gap-0.5 rounded-md px-1 py-1.5 text-[10px] leading-tight transition-colors"
				:class="[
					isActive(item.key)
						? 'text-sky-600 bg-sky-50 font-medium'
						: 'text-slate-500 hover:bg-slate-50 hover:text-slate-700',
				]"
				@click="navigate(item.to)"
			>
				<component :is="item.icon" class="h-4 w-4" />
				<span>{{ item.label }}</span>
			</button>
		</nav>

		<div class="flex flex-col items-center gap-0.5 px-1 pb-3">
			<button
				class="flex w-full flex-col items-center gap-0.5 rounded-md px-1 py-1.5 text-[10px] leading-tight text-slate-400 transition-colors hover:bg-slate-50 hover:text-slate-600"
				:title="isDark ? '切换到浅色模式' : '切换到深色模式'"
				@click="toggleTheme"
			>
				<Sun v-if="isDark" class="h-4 w-4" />
				<Moon v-else class="h-4 w-4" />
				<span>{{ isDark ? "浅色" : "深色" }}</span>
			</button>
			<button
				v-for="item in bottomItems"
				:key="item.key"
				class="flex w-full flex-col items-center gap-0.5 rounded-md px-1 py-1.5 text-[10px] leading-tight transition-colors"
				:class="[
					isActive(item.key)
						? 'text-sky-600 bg-sky-50 font-medium'
						: 'text-slate-400 hover:bg-slate-50 hover:text-slate-600',
				]"
				@click="navigate(item.to)"
			>
				<component :is="item.icon" class="h-4 w-4" />
				<span>{{ item.label }}</span>
			</button>
		</div>
	</aside>
</template>

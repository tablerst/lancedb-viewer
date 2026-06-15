<script setup lang="ts">
import { Database, Key, LayoutGrid, ListChecks, Search } from "lucide-vue-next"
import { computed } from "vue"
import { useRoute, useRouter } from "vue-router"

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

const items = computed<NavItem[]>(() => {
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
		{
			key: "capabilities",
			label: "能力",
			icon: ListChecks,
			to: "/capabilities",
		},
	]
})

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
	<aside class="flex h-full w-16 shrink-0 flex-col border-r border-slate-950 bg-[var(--app-nav)]">
		<div class="flex items-center justify-center border-b border-white/10 p-3">
			<div
				class="flex h-9 w-9 items-center justify-center rounded-md border border-sky-400/20 bg-sky-500/15 text-sky-200"
				title="LanceDB Studio"
				aria-label="LanceDB Studio"
			>
				<Database class="h-5 w-5" />
			</div>
		</div>

		<nav class="flex flex-1 flex-col items-center gap-1.5 px-2 py-4" aria-label="主导航">
			<NButton
				v-for="item in items"
				:key="item.key"
				size="small"
				quaternary
				circle
				:title="item.label"
				:aria-label="item.label"
				:class="[
					isActive(item.key)
						? 'bg-white/[0.12] text-white ring-1 ring-white/10'
						: 'text-slate-400 hover:bg-white/[0.08] hover:text-slate-100',
					'relative',
				]"
				@click="navigate(item.to)"
			>
				<span
					v-if="isActive(item.key)"
					class="absolute -left-2 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-full bg-sky-300"
				/>
				<component :is="item.icon" class="h-4 w-4" />
			</NButton>
		</nav>
	</aside>
</template>

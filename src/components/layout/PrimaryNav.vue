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
	<aside class="flex h-full w-14 shrink-0 flex-col border-r border-slate-200 bg-white">
		<div class="flex items-center justify-center p-3">
			<div
				class="flex h-9 w-9 items-center justify-center rounded-lg bg-sky-500 text-white"
				title="LanceDB Studio"
			>
				<Database class="h-5 w-5" />
			</div>
		</div>

		<div class="flex flex-1 flex-col items-center gap-2 px-2 pb-4">
			<NButton
				v-for="item in items"
				:key="item.key"
				size="small"
				quaternary
				circle
				:title="item.label"
				:class="[
					isActive(item.key) ? 'text-sky-600 bg-sky-50/80' : 'text-slate-600',
					'hover:bg-slate-50',
				]"
				@click="navigate(item.to)"
			>
				<component :is="item.icon" class="h-4 w-4" />
			</NButton>
		</div>
	</aside>
</template>

import { createRouter, createWebHashHistory } from "vue-router";

import ExplorerView from "./views/ExplorerView.vue";
import SearchView from "./views/SearchView.vue";

export const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{ path: "/", name: "explorer", component: ExplorerView },
		{ path: "/search", name: "search", component: SearchView },
	],
});

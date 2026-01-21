import { createRouter, createWebHashHistory } from "vue-router"

import ExplorerView from "./views/ExplorerView.vue"
import EditConnectionDialog from "./views/EditConnectionDialog.vue"
import NewConnectionDialog from "./views/NewConnectionDialog.vue"
import SearchView from "./views/SearchView.vue"

export const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{ path: "/", name: "explorer", component: ExplorerView },
		{ path: "/search", name: "search", component: SearchView },
		{
			path: "/dialog/new-connection",
			name: "new-connection-dialog",
			component: NewConnectionDialog,
			meta: { layout: "dialog" },
		},
		{
			path: "/dialog/edit-connection",
			name: "edit-connection-dialog",
			component: EditConnectionDialog,
			meta: { layout: "dialog" },
		},
	],
})

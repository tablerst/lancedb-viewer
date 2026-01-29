import { createRouter, createWebHashHistory } from "vue-router"

import CapabilitiesView from "./views/CapabilitiesView.vue"
import ConnectionCredentialsView from "./views/ConnectionCredentialsView.vue"
import CredentialsView from "./views/CredentialsView.vue"
import EditConnectionDialog from "./views/EditConnectionDialog.vue"
import ExplorerView from "./views/ExplorerView.vue"
import NewConnectionDialog from "./views/NewConnectionDialog.vue"
import SearchView from "./views/SearchView.vue"

export const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{ path: "/", name: "explorer", component: ExplorerView },
		{ path: "/connections/:id", name: "connection-explorer", component: ExplorerView },
		{ path: "/connections/:id/search", name: "connection-search", component: SearchView },
		{
			path: "/connections/:id/credentials",
			name: "connection-credentials",
			component: ConnectionCredentialsView,
		},
		{ path: "/search", name: "search", component: SearchView },
		{ path: "/vault/credentials", name: "vault-credentials", component: CredentialsView },
		{ path: "/credentials", redirect: "/vault/credentials" },
		{ path: "/capabilities", name: "capabilities", component: CapabilitiesView },
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

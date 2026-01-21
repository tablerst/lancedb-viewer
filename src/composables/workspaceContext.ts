import type { ComputedRef, Ref } from "vue"
import { inject, provide } from "vue"

import type { SchemaDefinition, TableInfo } from "../ipc/v1"
import type { StoredProfile } from "../models/profile"
import type { ConnectionState } from "./useConnection"

export interface ProfileFormState {
	name: string
	uri: string
	storageOptionsJson: string
}

export interface WorkspaceContext {
	profiles: Ref<StoredProfile[]>
	activeProfileId: Ref<string | null>
	activeProfile: ComputedRef<StoredProfile | null>
	profileForm: Ref<ProfileFormState>
	isSavingProfile: Ref<boolean>
	addProfile: () => Promise<void>
	updateProfile: (input: {
		id: string
		name: string
		uri: string
		storageOptionsJson: string
	}) => Promise<void>
	deleteProfile: (profileId: string) => Promise<void>
	setProfileLastConnected: (profileId: string, connectedAt: string) => Promise<void>
	selectProfile: (profileId: string) => Promise<void>

	connectionStates: Ref<Record<string, ConnectionState>>
	activeConnection: ComputedRef<ConnectionState | null>
	connectionId: ComputedRef<string | null>
	tables: ComputedRef<TableInfo[]>
	activeTableName: ComputedRef<string | null>
	activeTableId: ComputedRef<string | null>
	schema: ComputedRef<SchemaDefinition | null>
	isConnecting: ComputedRef<boolean>
	isRefreshing: ComputedRef<boolean>
	isOpening: ComputedRef<boolean>
	connectProfile: (profileId: string) => Promise<void>
	refreshTables: (profileId: string) => Promise<void>
	openTable: (profileId: string, name: string) => Promise<void>
	refreshSchema: (profileId: string) => Promise<void>
	resetConnection: (profileId: string) => void
	clearActiveTable: (profileId: string) => void

	statusMessage: Ref<string>
	errorMessage: Ref<string>
	setStatus: (message: string) => void
	setError: (message: string) => void
	clearMessages: () => void
}

const workspaceKey = Symbol("workspace-context")

export function provideWorkspace(context: WorkspaceContext) {
	provide(workspaceKey, context)
}

export function useWorkspace(): WorkspaceContext {
	const context = inject<WorkspaceContext>(workspaceKey)
	if (!context) {
		throw new Error("workspace context is not provided")
	}
	return context
}

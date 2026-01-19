import type { ComputedRef, Ref } from "vue";
import { inject, provide } from "vue";

import type { SchemaDefinition, TableInfo } from "../ipc/v1";
import type { StoredProfile } from "../models/profile";

export interface ProfileFormState {
	name: string;
	uri: string;
	storageOptionsJson: string;
}

export interface WorkspaceContext {
	profiles: Ref<StoredProfile[]>;
	activeProfileId: Ref<string | null>;
	activeProfile: ComputedRef<StoredProfile | null>;
	profileForm: Ref<ProfileFormState>;
	isSavingProfile: Ref<boolean>;
	addProfile: () => Promise<void>;
	selectProfile: (profileId: string) => Promise<void>;

	connectionId: Ref<string | null>;
	tables: Ref<TableInfo[]>;
	activeTableName: Ref<string | null>;
	activeTableId: Ref<string | null>;
	schema: Ref<SchemaDefinition | null>;
	isConnecting: Ref<boolean>;
	isRefreshing: Ref<boolean>;
	isOpening: Ref<boolean>;
	connectActiveProfile: () => Promise<void>;
	refreshTables: () => Promise<void>;
	openTable: (name: string) => Promise<void>;
	resetConnection: () => void;

	statusMessage: Ref<string>;
	errorMessage: Ref<string>;
	setStatus: (message: string) => void;
	setError: (message: string) => void;
	clearMessages: () => void;
}

const workspaceKey = Symbol("workspace-context");

export function provideWorkspace(context: WorkspaceContext) {
	provide(workspaceKey, context);
}

export function useWorkspace(): WorkspaceContext {
	const context = inject<WorkspaceContext>(workspaceKey);
	if (!context) {
		throw new Error("workspace context is not provided");
	}
	return context;
}

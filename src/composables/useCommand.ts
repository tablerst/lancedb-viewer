import { type Ref, ref } from "vue"
import { useWorkspace } from "./workspaceContext"

/**
 * Composable for executing async IPC commands with unified loading state and error handling.
 *
 * Manages `isLoading` state, calls `clearMessages()` before execution, and catches errors
 * to display via `setError()`. Prevents concurrent execution of the same command.
 *
 * @param errorMessage - Fallback message when the caught error is not an `Error` instance.
 */
export function useCommand(errorMessage = "操作失败"): {
	execute: <T>(action: () => Promise<T>) => Promise<T | undefined>
	isLoading: Ref<boolean>
} {
	const { setError, clearMessages } = useWorkspace()
	const isLoading = ref(false)

	async function execute<T>(action: () => Promise<T>): Promise<T | undefined> {
		if (isLoading.value) return undefined
		try {
			isLoading.value = true
			clearMessages()
			return await action()
		} catch (error) {
			setError(error instanceof Error ? error.message : errorMessage)
			return undefined
		} finally {
			isLoading.value = false
		}
	}

	return { execute, isLoading }
}

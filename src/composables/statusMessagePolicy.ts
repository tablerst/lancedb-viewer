export function shouldClearTransientMessagesOnRouteChange(previousFullPath: string | undefined) {
	return previousFullPath !== undefined
}

import { ref } from "vue"

export function useStatusMessages() {
	const statusMessage = ref("")
	const errorMessage = ref("")

	const setStatus = (message: string) => {
		statusMessage.value = message
		errorMessage.value = ""
	}

	const setError = (message: string) => {
		errorMessage.value = message
		statusMessage.value = ""
	}

	const clearMessages = () => {
		statusMessage.value = ""
		errorMessage.value = ""
	}

	return {
		statusMessage,
		errorMessage,
		setStatus,
		setError,
		clearMessages,
	}
}

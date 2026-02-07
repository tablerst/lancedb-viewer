<script setup lang="ts">
import { useMessage } from "naive-ui"
import { watch } from "vue"

const props = defineProps<{
	statusMessage: string
	errorMessage: string
}>()

const message = useMessage()

watch(
	() => props.statusMessage,
	(value, previous) => {
		if (!value || value === previous) {
			return
		}
		message.success(value, { duration: 3500 })
	}
)

// Errors are now shown as persistent NAlert in App.vue main area.
// Keep a subtle toast as secondary indicator only.
watch(
	() => props.errorMessage,
	(value, previous) => {
		if (!value || value === previous) {
			return
		}
		message.error(value, { duration: 5000 })
	}
)
</script>

<template>
	<span class="hidden" />
</template>

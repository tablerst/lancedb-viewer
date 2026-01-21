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
		message.success(value)
	}
)

watch(
	() => props.errorMessage,
	(value, previous) => {
		if (!value || value === previous) {
			return
		}
		message.error(value)
	}
)
</script>

<template>
	<span class="hidden" />
</template>

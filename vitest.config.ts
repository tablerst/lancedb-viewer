import { defineConfig } from "vitest/config"

export default defineConfig({
	test: {
		environment: "node",
		include: ["src/**/*.test.ts"],
		coverage: {
			provider: "v8",
			include: [
				"src/composables/statusMessagePolicy.ts",
				"src/views/explorer/mutationGuards.ts",
				"src/views/explorer/versionMetadata.ts",
				"src/views/search/searchRequests.ts",
			],
			all: true,
			thresholds: {
				lines: 80,
				functions: 80,
				branches: 80,
				statements: 80,
			},
		},
	},
})

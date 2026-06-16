import { describe, expect, it } from "vitest"
import { shouldClearTransientMessagesOnRouteChange } from "./statusMessagePolicy"

describe("status message route lifecycle policy", () => {
	it("keeps messages during the initial route binding", () => {
		expect(shouldClearTransientMessagesOnRouteChange(undefined)).toBe(false)
	})

	it("clears transient messages on later route changes", () => {
		expect(shouldClearTransientMessagesOnRouteChange("/connections/one/search")).toBe(true)
	})
})

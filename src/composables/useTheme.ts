import { ref, watch } from "vue"

export type ThemeMode = "light" | "dark" | "system"

const STORAGE_KEY = "lancedb-viewer-theme"

function getSystemPrefersDark(): boolean {
	return window.matchMedia("(prefers-color-scheme: dark)").matches
}

function applyThemeClass(isDark: boolean) {
	document.documentElement.classList.toggle("dark", isDark)
}

function loadStoredMode(): ThemeMode {
	try {
		const stored = localStorage.getItem(STORAGE_KEY)
		if (stored === "light" || stored === "dark" || stored === "system") {
			return stored
		}
	} catch {
		// localStorage unavailable
	}
	return "light"
}

const mode = ref<ThemeMode>(loadStoredMode())
const isDark = ref(false)

function updateDarkState() {
	isDark.value = mode.value === "dark" || (mode.value === "system" && getSystemPrefersDark())
	applyThemeClass(isDark.value)
}

// React to mode changes
watch(mode, (newMode) => {
	try {
		localStorage.setItem(STORAGE_KEY, newMode)
	} catch {
		// ignore
	}
	updateDarkState()
})

// Listen for OS theme changes
if (typeof window !== "undefined") {
	const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
	mediaQuery.addEventListener("change", () => {
		if (mode.value === "system") {
			updateDarkState()
		}
	})
}

// Apply initial state
updateDarkState()

export function useTheme() {
	return {
		/** Current theme mode preference */
		mode,
		/** Whether the resolved theme is dark */
		isDark,
		/** Toggle between light and dark (ignores system) */
		toggle() {
			mode.value = isDark.value ? "light" : "dark"
		},
	}
}

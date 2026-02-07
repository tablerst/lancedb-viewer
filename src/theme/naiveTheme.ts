import type { GlobalThemeOverrides } from "naive-ui"

export const themeOverrides: GlobalThemeOverrides = {
	common: {
		primaryColor: "#38bdf8",
		primaryColorHover: "#7dd3fc",
		primaryColorPressed: "#0ea5e9",
		primaryColorSuppl: "#38bdf8",
		successColor: "#14b8a6",
		successColorHover: "#2dd4bf",
		successColorPressed: "#0f766e",
		errorColor: "#ef4444",
		errorColorHover: "#f87171",
		errorColorPressed: "#dc2626",
		warningColor: "#f59e0b",
		warningColorHover: "#fbbf24",
		warningColorPressed: "#d97706",
		borderRadius: "10px",
		fontFamily: '"Inter", "Noto Sans SC", system-ui, sans-serif',
		textColor1: "#0f172a",
		textColor2: "#334155",
		textColor3: "#64748b",
		bodyColor: "#f8fafc",
	},
	Layout: {
		color: "#f1f5f9",
		headerColor: "#f8fafc",
		siderColor: "#f8fafc",
	},
	Card: {
		color: "#ffffff",
		borderColor: "#e2e8f0",
		borderRadius: "12px",
		paddingMedium: "20px",
		titleFontWeight: "600",
	},
	Button: {
		borderRadiusSmall: "8px",
		borderRadiusMedium: "10px",
	},
	Input: {
		borderRadius: "10px",
	},
	DataTable: {
		borderRadius: "10px",
	},
	Tag: {
		borderRadius: "8px",
	},
}

/** Dark mode overrides — applied when `<html class="dark">` is set */
export const darkThemeOverrides: GlobalThemeOverrides = {
	common: {
		primaryColor: "#38bdf8",
		primaryColorHover: "#7dd3fc",
		primaryColorPressed: "#0ea5e9",
		primaryColorSuppl: "#38bdf8",
		successColor: "#14b8a6",
		successColorHover: "#2dd4bf",
		successColorPressed: "#0f766e",
		errorColor: "#ef4444",
		errorColorHover: "#f87171",
		errorColorPressed: "#dc2626",
		warningColor: "#f59e0b",
		warningColorHover: "#fbbf24",
		warningColorPressed: "#d97706",
		borderRadius: "10px",
		fontFamily: '"Inter", "Noto Sans SC", system-ui, sans-serif',
		bodyColor: "#0f172a",
		cardColor: "#1e293b",
		modalColor: "#1e293b",
		popoverColor: "#1e293b",
		tableColor: "#1e293b",
		inputColor: "#1e293b",
		textColor1: "#f1f5f9",
		textColor2: "#cbd5e1",
		textColor3: "#94a3b8",
		borderColor: "#334155",
		dividerColor: "#334155",
		hoverColor: "rgba(255,255,255,0.06)",
	},
	Card: {
		color: "#1e293b",
		borderColor: "#334155",
		borderRadius: "12px",
		paddingMedium: "20px",
		titleFontWeight: "600",
	},
	Layout: {
		color: "#0f172a",
		headerColor: "#0f172a",
		siderColor: "#0f172a",
	},
	Button: {
		borderRadiusSmall: "8px",
		borderRadiusMedium: "10px",
	},
	Input: {
		borderRadius: "10px",
	},
	DataTable: {
		borderRadius: "10px",
	},
	Tag: {
		borderRadius: "8px",
	},
}

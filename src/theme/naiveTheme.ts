import type { GlobalThemeOverrides } from "naive-ui"

export const themeOverrides: GlobalThemeOverrides = {
	common: {
		primaryColor: "#0284c7",
		primaryColorHover: "#0369a1",
		primaryColorPressed: "#075985",
		primaryColorSuppl: "#0284c7",
		successColor: "#14b8a6",
		successColorHover: "#2dd4bf",
		successColorPressed: "#0f766e",
		errorColor: "#ef4444",
		errorColorHover: "#f87171",
		errorColorPressed: "#dc2626",
		warningColor: "#f59e0b",
		warningColorHover: "#fbbf24",
		warningColorPressed: "#d97706",
		borderRadius: "8px",
		fontFamily: '"Inter", "Noto Sans SC", system-ui, sans-serif',
		textColor1: "#0f172a",
		textColor2: "#334155",
		textColor3: "#64748b",
		bodyColor: "#f6f8fb",
	},
	Layout: {
		color: "#f6f8fb",
		headerColor: "#f6f8fb",
		siderColor: "#f6f8fb",
	},
	Card: {
		color: "#fbfdff",
		borderColor: "#dbe3ee",
		borderRadius: "8px",
		paddingMedium: "16px",
	},
	Button: {
		borderRadiusSmall: "8px",
		borderRadiusMedium: "8px",
	},
	Input: {
		borderRadius: "8px",
	},
	DataTable: {
		borderRadius: "8px",
	},
	Tag: {
		borderRadius: "6px",
	},
}

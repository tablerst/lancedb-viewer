import type { GlobalThemeOverrides } from "naive-ui";

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
		fontFamily: "\"Inter\", \"Noto Sans SC\", system-ui, sans-serif",
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
};

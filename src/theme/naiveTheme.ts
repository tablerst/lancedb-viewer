import type { GlobalThemeOverrides } from "naive-ui"

export const themeOverrides: GlobalThemeOverrides = {
	common: {
		primaryColor: "#5e6ad2",
		primaryColorHover: "#6f78df",
		primaryColorPressed: "#4f5bd5",
		primaryColorSuppl: "#5e6ad2",
		successColor: "#24855d",
		successColorHover: "#2f9469",
		successColorPressed: "#1f704f",
		errorColor: "#c52828",
		errorColorHover: "#d33a3a",
		errorColorPressed: "#a92323",
		warningColor: "#9a6700",
		warningColorHover: "#ad7608",
		warningColorPressed: "#805400",
		borderRadius: "6px",
		fontFamily: '"Inter", "Noto Sans SC", system-ui, sans-serif',
		textColor1: "#202124",
		textColor2: "#4b4d55",
		textColor3: "#6f7178",
		bodyColor: "#f7f7f4",
		borderColor: "#dfdfda",
		dividerColor: "#dfdfda",
		hoverColor: "#ececea",
	},
	Layout: {
		color: "#f7f7f4",
		headerColor: "#f7f7f4",
		siderColor: "#f1f1ee",
	},
	Card: {
		color: "#ffffff",
		borderColor: "#dfdfda",
		borderRadius: "8px",
		paddingMedium: "16px",
		titleFontWeight: "600",
	},
	Button: {
		borderRadiusSmall: "5px",
		borderRadiusMedium: "6px",
	},
	Input: {
		borderRadius: "6px",
	},
	DataTable: {
		borderRadius: "6px",
	},
	Tag: {
		borderRadius: "5px",
	},
}

/** Dark mode overrides — applied when `<html class="dark">` is set */
export const darkThemeOverrides: GlobalThemeOverrides = {
	common: {
		primaryColor: "#8f96ff",
		primaryColorHover: "#aeb3ff",
		primaryColorPressed: "#747eff",
		primaryColorSuppl: "#8f96ff",
		successColor: "#61c083",
		successColorHover: "#72cf93",
		successColorPressed: "#4aa66c",
		errorColor: "#ff6b6b",
		errorColorHover: "#ff8585",
		errorColorPressed: "#e45454",
		warningColor: "#d4a245",
		warningColorHover: "#e4b457",
		warningColorPressed: "#b48638",
		borderRadius: "6px",
		fontFamily: '"Inter", "Noto Sans SC", system-ui, sans-serif',
		bodyColor: "#0f1012",
		cardColor: "#1b1c20",
		modalColor: "#1b1c20",
		popoverColor: "#1b1c20",
		tableColor: "#1b1c20",
		inputColor: "#151619",
		textColor1: "#e7e7ea",
		textColor2: "#c4c6cc",
		textColor3: "#a4a6ad",
		borderColor: "#2d2f36",
		dividerColor: "#2d2f36",
		hoverColor: "rgba(255,255,255,0.055)",
	},
	Card: {
		color: "#1b1c20",
		borderColor: "#2d2f36",
		borderRadius: "8px",
		paddingMedium: "16px",
		titleFontWeight: "600",
	},
	Layout: {
		color: "#0f1012",
		headerColor: "#0f1012",
		siderColor: "#151619",
	},
	Button: {
		borderRadiusSmall: "5px",
		borderRadiusMedium: "6px",
	},
	Input: {
		borderRadius: "6px",
	},
	DataTable: {
		borderRadius: "6px",
	},
	Tag: {
		borderRadius: "5px",
	},
}

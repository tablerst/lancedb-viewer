export type ConnectionKind = "local" | "s3" | "gcs" | "azure" | "remote" | "unknown"

export function getConnectionKind(uri: string): ConnectionKind {
	const normalized = uri.trim().toLowerCase()
	if (normalized.startsWith("file://")) {
		return "local"
	}
	if (normalized.startsWith("s3://")) {
		return "s3"
	}
	if (normalized.startsWith("s3+ddb://")) {
		return "s3"
	}
	if (normalized.startsWith("gs://")) {
		return "gcs"
	}
	if (normalized.startsWith("az://") || normalized.startsWith("azure://")) {
		return "azure"
	}
	if (normalized.startsWith("db://")) {
		return "remote"
	}
	if (normalized.includes("://")) {
		return "unknown"
	}
	return "local"
}

export function getConnectionKindLabel(kind: ConnectionKind) {
	switch (kind) {
		case "local":
			return "Local"
		case "s3":
			return "S3"
		case "gcs":
			return "GCS"
		case "azure":
			return "Azure"
		case "remote":
			return "Remote"
		default:
			return "Unknown"
	}
}

export function getConnectionKindTagType(kind: ConnectionKind) {
	switch (kind) {
		case "local":
			return "success"
		case "s3":
		case "gcs":
		case "azure":
			return "info"
		case "remote":
			return "warning"
		default:
			return "default"
	}
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Local,
    S3,
    Gcs,
    Azure,
    Remote,
    Unknown,
}

pub fn infer_backend_kind(uri: &str) -> BackendKind {
    let lower = uri.trim().to_lowercase();
    if lower.starts_with("s3://") || lower.starts_with("s3+ddb://") {
        BackendKind::S3
    } else if lower.starts_with("gs://") {
        BackendKind::Gcs
    } else if lower.starts_with("az://") {
        BackendKind::Azure
    } else if lower.starts_with("db://") {
        BackendKind::Remote
    } else if lower.contains("://") {
        BackendKind::Unknown
    } else {
        BackendKind::Local
    }
}

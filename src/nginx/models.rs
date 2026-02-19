use serde::{Deserialize, Serialize};

/// Nginx proxy configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct NginxProxy {
    pub name: String,
    pub domain: String,
    pub backend: String,
    pub ssl: bool,
    pub extra_config: Option<String>,
}

/// Generic nginx operation response
#[derive(Serialize)]
pub struct NginxResponse {
    pub success: bool,
    pub message: String,
}

/// Request for config formatting
#[derive(Serialize, Deserialize)]
pub struct FormatRequest {
    pub config: String,
}

/// Formatted config response
#[derive(Serialize)]
pub struct FormatResponse {
    pub success: bool,
    pub formatted: Option<String>,
    pub error: Option<String>,
}

//! Error handling module
//!
//! This module provides consistent error types and handling across all application modules.
//! All errors are converted to HTTP responses with appropriate status codes and JSON formatting.
//!
//! # Error Types
//!
//! - [`AppError::SystemError`]: System monitoring errors (CPU, memory, disk, network, processes)
//! - [`AppError::NginxError`]: Nginx proxy management errors
//! - [`AppError::DockerError`]: Docker management errors
//! - [`AppError::ConfigError`]: Configuration loading errors
//! - [`AppError::NotFound`]: Resource not found errors (404)
//! - [`AppError::ValidationError`]: Input validation errors (400)
//!
//! # Error Response Format
//!
//! All errors are returned as JSON with a consistent structure:
//!
//! ```json
//! {
//!     "error": "Error Type",
//!     "message": "Detailed error message"
//! }
//! ```
//!
//! # Example
//!
//! ```
//! use ubuntu_resource_api::error::AppError;
//! use actix_web::{error::ResponseError, HttpResponse};
//!
//! fn example_handler() -> Result<HttpResponse, AppError> {
//!     // Return an error that will be automatically converted to HTTP response
//!     Err(AppError::NotFound("Resource not found".to_string()))
//! }
//! ```

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;
use std::fmt;

/// Common error types for the application
#[derive(Debug)]
pub enum AppError {
    /// System-related errors (CPU, memory, disk, network, processes)
    SystemError(String),
    /// Nginx proxy management errors
    NginxError(String),
    /// Docker management errors
    DockerError(String),
    /// Configuration loading errors
    ConfigError(String),
    /// Resource not found errors
    NotFound(String),
    /// Input validation errors
    ValidationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::SystemError(msg) => write!(f, "System Error: {}", msg),
            AppError::NginxError(msg) => write!(f, "Nginx Error: {}", msg),
            AppError::DockerError(msg) => write!(f, "Docker Error: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Configuration Error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(msg) => HttpResponse::NotFound().json(json!({
                "error": "Not Found",
                "message": msg
            })),
            AppError::ValidationError(msg) => HttpResponse::BadRequest().json(json!({
                "error": "Validation Error",
                "message": msg
            })),
            AppError::SystemError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": "System Error",
                "message": msg
            })),
            AppError::NginxError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": "Nginx Error",
                "message": msg
            })),
            AppError::DockerError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": "Docker Error",
                "message": msg
            })),
            AppError::ConfigError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": "Configuration Error",
                "message": msg
            })),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::SystemError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NginxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DockerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        // Test NotFound returns 404
        let err = AppError::NotFound("Resource not found".to_string());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);

        // Test ValidationError returns 400
        let err = AppError::ValidationError("Invalid input".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);

        // Test SystemError returns 500
        let err = AppError::SystemError("System failure".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Test NginxError returns 500
        let err = AppError::NginxError("Nginx failure".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Test DockerError returns 500
        let err = AppError::DockerError("Docker failure".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Test ConfigError returns 500
        let err = AppError::ConfigError("Config failure".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        let err = AppError::NotFound("test resource".to_string());
        assert_eq!(format!("{}", err), "Not Found: test resource");

        let err = AppError::ValidationError("test validation".to_string());
        assert_eq!(format!("{}", err), "Validation Error: test validation");

        let err = AppError::SystemError("test system".to_string());
        assert_eq!(format!("{}", err), "System Error: test system");

        let err = AppError::NginxError("test nginx".to_string());
        assert_eq!(format!("{}", err), "Nginx Error: test nginx");

        let err = AppError::DockerError("test docker".to_string());
        assert_eq!(format!("{}", err), "Docker Error: test docker");

        let err = AppError::ConfigError("test config".to_string());
        assert_eq!(format!("{}", err), "Configuration Error: test config");
    }

    #[test]
    fn test_error_response_structure() {
        // Test that error responses contain the expected JSON structure
        let err = AppError::NotFound("test".to_string());
        let response = err.error_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let err = AppError::ValidationError("test".to_string());
        let response = err.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let err = AppError::SystemError("test".to_string());
        let response = err.error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

//! Nginx proxy management module
//!
//! This module provides functionality for managing nginx reverse proxy configurations:
//! - Create, read, update, and delete nginx proxy configurations
//! - Generate nginx configuration files
//! - Validate nginx configuration syntax
//! - Format nginx configuration for readability
//!
//! # Submodules
//!
//! - [`models`]: Data structures for nginx proxy configurations
//! - [`config`]: Nginx configuration generation and validation utilities
//! - [`handlers`]: HTTP request handlers for nginx management endpoints
//! - [`routes`]: Route registration for nginx management API
//!
//! # Example
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer};
//! use ubuntu_resource_api::nginx;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .configure(nginx::routes::configure_routes)
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```

pub mod models;
pub mod config;
pub mod handlers;
pub mod routes;

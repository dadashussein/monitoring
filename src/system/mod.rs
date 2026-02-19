//! System monitoring module
//!
//! This module provides functionality for monitoring system resources including:
//! - CPU information and usage statistics
//! - Memory usage (total, used, free, available)
//! - Disk usage for all mounted filesystems
//! - Network interface statistics
//! - Running processes with CPU/memory usage
//! - System load average
//! - Process management (kill processes)
//!
//! # Submodules
//!
//! - [`models`]: Data structures for system information
//! - [`handlers`]: HTTP request handlers for system monitoring endpoints
//! - [`routes`]: Route registration for system monitoring API
//!
//! # Example
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer};
//! use ubuntu_resource_api::system;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .configure(system::configure_routes)
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```

pub mod models;
pub mod handlers;
pub mod routes;

pub use models::*;
pub use routes::configure_routes;

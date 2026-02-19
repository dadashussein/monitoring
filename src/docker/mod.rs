//! Docker management module
//!
//! This module provides functionality for managing Docker resources:
//! - List, start, stop, restart, and remove containers
//! - View container logs
//! - List and remove images
//! - List and remove volumes
//! - List and remove networks
//!
//! # Submodules
//!
//! - [`models`]: Data structures for Docker entities (containers, images, volumes, networks)
//! - [`client`]: Docker client initialization and utilities
//! - [`handlers`]: HTTP request handlers for Docker management endpoints
//! - [`routes`]: Route registration for Docker management API
//!
//! # Example
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer};
//! use ubuntu_resource_api::docker;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .configure(docker::routes::configure_routes)
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```

pub mod models;
pub mod client;
pub mod handlers;
pub mod routes;

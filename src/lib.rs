//! Ubuntu Resource Monitor Library
//!
//! This library provides a modular REST API for monitoring system resources,
//! managing nginx proxy configurations, and controlling Docker containers.
//!
//! # Architecture
//!
//! The application is organized into feature modules:
//!
//! - [`system`]: System monitoring (CPU, memory, disk, network, processes)
//! - [`nginx`]: Nginx proxy management (CRUD operations, config generation)
//! - [`docker`]: Docker management (containers, images, volumes, networks)
//! - [`config`]: Configuration management with environment variable support
//! - [`error`]: Common error types and consistent error handling
//! - [`utils`]: Shared utility functions
//!
//! # Example
//!
//! ```no_run
//! use ubuntu_resource_api::config::AppConfig;
//! use ubuntu_resource_api::system;
//! use ubuntu_resource_api::nginx;
//! use ubuntu_resource_api::docker;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     // Load configuration
//!     let config = AppConfig::from_env()
//!         .unwrap_or_else(|_| AppConfig::with_defaults());
//!     
//!     // Configure routes and start server
//!     // (see main.rs for complete example)
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod utils;
pub mod system;
pub mod nginx;
pub mod docker;

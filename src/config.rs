//! Configuration management module
//!
//! This module provides centralized configuration management for the application.
//! Configuration can be loaded from environment variables or use sensible defaults.
//!
//! # Environment Variables
//!
//! - `SERVER_BIND_ADDRESS`: Server bind address and port (default: `0.0.0.0:8080`)
//! - `NGINX_SITES_AVAILABLE`: Nginx sites-available directory (default: `/etc/nginx/sites-available`)
//! - `NGINX_SITES_ENABLED`: Nginx sites-enabled directory (default: `/etc/nginx/sites-enabled`)
//! - `DOCKER_SOCKET_PATH`: Docker socket path (default: `unix:///var/run/docker.sock`)
//!
//! # Example
//!
//! ```
//! use ubuntu_resource_api::config::AppConfig;
//!
//! // Load from environment variables with fallback to defaults
//! let config = AppConfig::from_env()
//!     .unwrap_or_else(|_| AppConfig::with_defaults());
//!
//! println!("Server will bind to: {}", config.server.bind_address);
//! ```

use std::env;

/// Main application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub nginx: NginxConfig,
    pub docker: DockerConfig,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
}

/// Nginx configuration
#[derive(Debug, Clone)]
pub struct NginxConfig {
    pub sites_available_path: String,
    pub sites_enabled_path: String,
}

/// Docker configuration
#[derive(Debug, Clone)]
pub struct DockerConfig {
    pub socket_path: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    /// Returns an error if required variables are missing or invalid
    pub fn from_env() -> Result<Self, String> {
        let server = ServerConfig {
            bind_address: env::var("SERVER_BIND_ADDRESS")
                .unwrap_or_else(|_| Self::default_bind_address()),
        };

        let nginx = NginxConfig {
            sites_available_path: env::var("NGINX_SITES_AVAILABLE")
                .unwrap_or_else(|_| Self::default_nginx_sites_available()),
            sites_enabled_path: env::var("NGINX_SITES_ENABLED")
                .unwrap_or_else(|_| Self::default_nginx_sites_enabled()),
        };

        let docker = DockerConfig {
            socket_path: env::var("DOCKER_SOCKET_PATH")
                .unwrap_or_else(|_| Self::default_docker_socket()),
        };

        Ok(AppConfig {
            server,
            nginx,
            docker,
        })
    }

    /// Create configuration with default values
    pub fn with_defaults() -> Self {
        AppConfig {
            server: ServerConfig {
                bind_address: Self::default_bind_address(),
            },
            nginx: NginxConfig {
                sites_available_path: Self::default_nginx_sites_available(),
                sites_enabled_path: Self::default_nginx_sites_enabled(),
            },
            docker: DockerConfig {
                socket_path: Self::default_docker_socket(),
            },
        }
    }

    // Default value helpers
    fn default_bind_address() -> String {
        "0.0.0.0:8080".to_string()
    }

    fn default_nginx_sites_available() -> String {
        "/etc/nginx/sites-available".to_string()
    }

    fn default_nginx_sites_enabled() -> String {
        "/etc/nginx/sites-enabled".to_string()
    }

    fn default_docker_socket() -> String {
        "unix:///var/run/docker.sock".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Use a mutex to ensure tests run serially to avoid env var conflicts
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_with_defaults() {
        let config = AppConfig::with_defaults();
        
        assert_eq!(config.server.bind_address, "0.0.0.0:8080");
        assert_eq!(config.nginx.sites_available_path, "/etc/nginx/sites-available");
        assert_eq!(config.nginx.sites_enabled_path, "/etc/nginx/sites-enabled");
        assert_eq!(config.docker.socket_path, "unix:///var/run/docker.sock");
    }

    #[test]
    fn test_from_env_with_defaults() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Clear any existing env vars
        env::remove_var("SERVER_BIND_ADDRESS");
        env::remove_var("NGINX_SITES_AVAILABLE");
        env::remove_var("NGINX_SITES_ENABLED");
        env::remove_var("DOCKER_SOCKET_PATH");

        let config = AppConfig::from_env().unwrap();
        
        assert_eq!(config.server.bind_address, "0.0.0.0:8080");
        assert_eq!(config.nginx.sites_available_path, "/etc/nginx/sites-available");
        assert_eq!(config.nginx.sites_enabled_path, "/etc/nginx/sites-enabled");
        assert_eq!(config.docker.socket_path, "unix:///var/run/docker.sock");
    }

    #[test]
    fn test_from_env_with_custom_values() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Set custom env vars
        env::set_var("SERVER_BIND_ADDRESS", "127.0.0.1:9000");
        env::set_var("NGINX_SITES_AVAILABLE", "/custom/nginx/available");
        env::set_var("NGINX_SITES_ENABLED", "/custom/nginx/enabled");
        env::set_var("DOCKER_SOCKET_PATH", "tcp://localhost:2375");

        let config = AppConfig::from_env().unwrap();
        
        assert_eq!(config.server.bind_address, "127.0.0.1:9000");
        assert_eq!(config.nginx.sites_available_path, "/custom/nginx/available");
        assert_eq!(config.nginx.sites_enabled_path, "/custom/nginx/enabled");
        assert_eq!(config.docker.socket_path, "tcp://localhost:2375");

        // Clean up
        env::remove_var("SERVER_BIND_ADDRESS");
        env::remove_var("NGINX_SITES_AVAILABLE");
        env::remove_var("NGINX_SITES_ENABLED");
        env::remove_var("DOCKER_SOCKET_PATH");
    }
}

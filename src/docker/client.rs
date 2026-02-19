// Docker client utilities

use bollard::Docker;

/// Initialize and return a Docker client connected to the specified socket path
pub async fn get_docker_client(socket_path: &str) -> Result<Docker, String> {
    Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
        .map_err(|e| format!("Docker connection failed: {}. Is Docker running?", e))
}

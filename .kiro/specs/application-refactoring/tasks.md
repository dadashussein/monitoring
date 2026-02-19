# Implementation Plan: Application Refactoring

## Overview

This plan breaks down the refactoring of the Ubuntu Resource Monitor application into incremental, testable steps. Each task builds on previous work, ensuring the application remains functional throughout the refactoring process. The approach follows the migration strategy defined in the design document, moving from structure creation through model extraction, handler separation, and finally configuration management.

## Tasks

- [ ] 1. Create foundational module structure and shared utilities
  - [x] 1.1 Create directory structure and empty module files
    - Create directories: src/system/, src/nginx/, src/docker/, src/templates/
    - Create empty mod.rs files in each feature directory
    - Create src/lib.rs with module declarations
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  
  - [x] 1.2 Create error handling module
    - Create src/error.rs with AppError enum
    - Implement ResponseError trait for AppError
    - Add error-to-HTTP status code mapping
    - _Requirements: 9.1, 9.2_
  
  - [x] 1.3 Create utilities module
    - Create src/utils.rs
    - Move bytes_to_gb function to utils.rs
    - Move refresh_system function to utils.rs
    - _Requirements: 5.1, 5.2, 5.3_
  
  - [ ]* 1.4 Write property test for utility function purity
    - **Property 1: Utility Function Purity**
    - **Validates: Requirements 5.5**
    - Test that bytes_to_gb produces consistent outputs for same inputs
    - _Requirements: 5.5_

- [ ] 2. Extract and organize system monitoring module
  - [x] 2.1 Create system module models
    - Create src/system/models.rs
    - Move all system-related structs (AppState, SystemInfo, CpuInfo, CpuUsage, MemoryInfo, DiskInfo, NetworkInfo, ProcessInfo, LoadAverage, HealthResponse, ApiInfo, ProcessQuery, KillResponse)
    - Add necessary derives and imports
    - Update src/system/mod.rs to export models
    - _Requirements: 2.1, 2.5_
  
  - [x] 2.2 Create system module handlers
    - Create src/system/handlers.rs
    - Move all system handler functions (index, get_system_info, get_cpu_info, get_cpu_usage, get_memory_info, get_disks_info, get_network_info, get_processes, get_load_average, health_check, kill_process, dashboard)
    - Update imports to use new module paths (utils, models, error)
    - Convert error handling to use AppError
    - _Requirements: 2.2, 2.3_
  
  - [x] 2.3 Create system module route registration
    - Create src/system/routes.rs
    - Implement configure_routes function
    - Register all system monitoring routes
    - Export configure_routes from src/system/mod.rs
    - _Requirements: 8.1, 8.2, 8.5_
  
  - [ ]* 2.4 Write unit tests for system module structure
    - Test that system/models.rs contains expected structs
    - Test that system/handlers.rs contains expected functions
    - Test that system/routes.rs exports configure_routes
    - _Requirements: 2.1, 2.2, 2.3_

- [ ] 3. Extract and organize nginx management module
  - [x] 3.1 Create nginx module models
    - Create src/nginx/models.rs
    - Move nginx-related structs (NginxProxy, NginxResponse, FormatRequest, FormatResponse)
    - Add necessary derives and imports
    - Update src/nginx/mod.rs to export models
    - _Requirements: 3.1_
  
  - [x] 3.2 Create nginx configuration utilities
    - Create src/nginx/config.rs
    - Move format_nginx_config function
    - Move validate_nginx_extra_config function
    - Move generate_nginx_config function
    - _Requirements: 3.3, 3.4_
  
  - [x] 3.3 Create nginx module handlers
    - Create src/nginx/handlers.rs
    - Move nginx handler functions (nginx_admin, format_nginx_extra_config, get_nginx_proxies, create_nginx_proxy, delete_nginx_proxy, update_nginx_proxy)
    - Update imports to use new module paths
    - Convert error handling to use AppError
    - _Requirements: 3.2, 3.5_
  
  - [x] 3.4 Create nginx module route registration
    - Create src/nginx/routes.rs
    - Implement configure_routes function
    - Register all nginx management routes
    - Export configure_routes from src/nginx/mod.rs
    - _Requirements: 8.1, 8.3, 8.5_
  
  - [ ]* 3.5 Write unit tests for nginx module structure
    - Test that nginx/models.rs contains expected structs
    - Test that nginx/config.rs contains config functions
    - Test that nginx/handlers.rs contains expected functions
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 4. Extract and organize docker management module
  - [x] 4.1 Create docker module models
    - Create src/docker/models.rs
    - Move Docker-related structs (DockerContainer, DockerImage, DockerVolume, DockerNetwork, DockerResponse, DockerLogsResponse)
    - Add necessary derives and imports
    - Update src/docker/mod.rs to export models
    - _Requirements: 4.1_
  
  - [x] 4.2 Create docker client utilities
    - Create src/docker/client.rs
    - Move get_docker_client function
    - _Requirements: 4.3_
  
  - [x] 4.3 Create docker module handlers
    - Create src/docker/handlers.rs
    - Move docker handler functions (docker_manager, list_containers, start_container, stop_container, restart_container, remove_container, get_container_logs, list_images, remove_image, list_volumes, remove_volume, list_networks, remove_network)
    - Update imports to use new module paths
    - Convert error handling to use AppError
    - _Requirements: 4.2, 4.4, 4.5_
  
  - [x] 4.4 Create docker module route registration
    - Create src/docker/routes.rs
    - Implement configure_routes function
    - Register all docker management routes
    - Export configure_routes from src/docker/mod.rs
    - _Requirements: 8.1, 8.4, 8.5_
  
  - [ ]* 4.5 Write unit tests for docker module structure
    - Test that docker/models.rs contains expected structs
    - Test that docker/client.rs contains get_docker_client
    - Test that docker/handlers.rs contains expected functions
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 5. Checkpoint - Verify module structure compilation
  - Ensure all modules compile successfully
  - Verify no duplicate code remains in main.rs
  - Ask the user if questions arise

- [ ] 6. Implement configuration management
  - [x] 6.1 Create configuration module
    - Create src/config.rs
    - Define AppConfig, ServerConfig, NginxConfig, DockerConfig structs
    - Implement AppConfig::from_env() to load from environment variables
    - Implement AppConfig::with_defaults() for default values
    - _Requirements: 6.1, 6.5_
  
  - [x] 6.2 Replace hard-coded server bind address
    - Update main.rs to load bind address from config
    - Use config.server.bind_address instead of hard-coded "0.0.0.0:8080"
    - _Requirements: 6.2_
  
  - [x] 6.3 Replace hard-coded nginx paths
    - Update nginx handlers to use config paths
    - Pass nginx config paths from main.rs to nginx module
    - Replace hard-coded /etc/nginx/sites-available and /etc/nginx/sites-enabled
    - _Requirements: 6.3_
  
  - [x] 6.4 Replace hard-coded docker socket path
    - Update docker client to use config socket path
    - Pass docker config from main.rs to docker module
    - _Requirements: 6.4_
  
  - [ ]* 6.5 Write unit tests for configuration loading
    - Test loading from environment variables
    - Test default value fallback
    - Test that all hard-coded values are replaced
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 7. Organize HTML dashboard files
  - [x] 7.1 Move HTML files to templates directory
    - Move src/dashboard.html to src/templates/dashboard.html
    - Move src/nginx_admin.html to src/templates/nginx_admin.html
    - Move src/docker_manager.html to src/templates/docker_manager.html
    - _Requirements: 7.1_
  
  - [x] 7.2 Update handlers to load HTML from templates directory
    - Update system::handlers::dashboard to load from templates/dashboard.html
    - Update nginx::handlers::nginx_admin to load from templates/nginx_admin.html
    - Update docker::handlers::docker_manager to load from templates/docker_manager.html
    - Use include_str! macro for compile-time embedding
    - _Requirements: 7.2, 7.3_
  
  - [ ]* 7.3 Write property test for dashboard route consistency
    - **Property 2: Dashboard Route Consistency**
    - **Validates: Requirements 7.4**
    - Test that all dashboard routes return correct HTML content
    - _Requirements: 7.4, 7.5_

- [ ] 8. Update main.rs to use modular structure
  - [x] 8.1 Simplify main.rs
    - Remove all moved code from main.rs
    - Keep only: logging initialization, config loading, AppState creation, server startup
    - Import modules from lib.rs
    - Call configure_routes for each module (system, nginx, docker)
    - Verify main.rs is under 100 lines
    - _Requirements: 1.5, 8.5_
  
  - [ ]* 8.2 Write unit test for main.rs structure
    - Test that main.rs is minimal (< 100 lines)
    - Test that main.rs calls all route registration functions
    - _Requirements: 1.5, 8.5_

- [x] 9. Checkpoint - Verify application functionality
  - Build and run the refactored application
  - Manually test key endpoints from each module
  - Ensure all tests pass
  - Ask the user if questions arise

- [ ] 10. Implement backward compatibility validation
  - [ ]* 10.1 Write property test for error response consistency
    - **Property 3: Error Response Consistency**
    - **Validates: Requirements 9.2, 9.3**
    - Test that all errors use AppError and consistent JSON format
    - _Requirements: 9.2, 9.3_
  
  - [ ]* 10.2 Write property test for error backward compatibility
    - **Property 4: Error Backward Compatibility**
    - **Validates: Requirements 9.4, 9.5**
    - Test that error messages and status codes match original
    - _Requirements: 9.4, 9.5_
  
  - [ ]* 10.3 Write property test for complete API backward compatibility
    - **Property 5: Complete API Backward Compatibility**
    - **Validates: Requirements 2.4, 3.5, 4.5, 10.2, 10.4**
    - Test that all API endpoints return identical responses to original
    - Test system monitoring endpoints
    - Test nginx management endpoints
    - Test docker management endpoints
    - _Requirements: 2.4, 3.5, 4.5, 10.1, 10.2, 10.3, 10.4_

- [ ] 11. Integration testing and validation
  - [ ]* 11.1 Write integration tests for system monitoring API
    - Test complete flows: get system info, get CPU info, get processes, kill process
    - Verify responses match expected formats
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  
  - [ ]* 11.2 Write integration tests for nginx management API
    - Test complete flows: list proxies, create proxy, update proxy, delete proxy
    - Verify nginx config generation and validation
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  
  - [ ]* 11.3 Write integration tests for docker management API
    - Test complete flows: list containers, start/stop/restart container, list images
    - Verify docker client operations
    - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 12. Final checkpoint and documentation
  - [x] 12.1 Update project documentation
    - Update README.md with new module structure
    - Document configuration options and environment variables
    - Add module-level documentation comments
    - _Requirements: All_
  
  - [x] 12.2 Final validation
    - Run all unit tests
    - Run all property tests
    - Run all integration tests
    - Verify no compilation warnings
    - Ensure all tests pass, ask the user if questions arise

## Notes

- Tasks marked with `*` are optional testing tasks that can be skipped for faster completion
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation throughout the refactoring
- The refactoring maintains backward compatibility at every step
- Property tests validate universal correctness properties
- Unit tests validate specific structural requirements
- Integration tests validate end-to-end functionality

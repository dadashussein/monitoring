# Design Document: Application Refactoring

## Overview

This design describes the refactoring of the Ubuntu Resource Monitor application from a monolithic ~1750-line main.rs file into a well-structured, modular Rust application. The refactoring will organize code by domain (system monitoring, nginx management, Docker management) while maintaining all existing functionality and API compatibility.

The refactored application will follow Rust best practices with a clear module hierarchy, separated concerns, and externalized configuration. The architecture will improve maintainability, testability, and make future feature additions easier.

## Architecture

### Module Structure

The application will be reorganized into the following module hierarchy:

```
src/
├── main.rs                 # Application entry point and server initialization
├── lib.rs                  # Library root exposing public modules
├── config.rs               # Configuration management
├── error.rs                # Common error types and handling
├── utils.rs                # Shared utility functions
├── templates/              # HTML dashboard files
│   ├── dashboard.html
│   ├── nginx_admin.html
│   └── docker_manager.html
├── system/                 # System monitoring module
│   ├── mod.rs             # Module declaration
│   ├── models.rs          # System info data structures
│   ├── handlers.rs        # HTTP request handlers
│   └── routes.rs          # Route registration
├── nginx/                  # Nginx management module
│   ├── mod.rs             # Module declaration
│   ├── models.rs          # Nginx proxy data structures
│   ├── handlers.rs        # HTTP request handlers
│   ├── config.rs          # Nginx config generation and validation
│   └── routes.rs          # Route registration
└── docker/                 # Docker management module
    ├── mod.rs             # Module declaration
    ├── models.rs          # Docker entity data structures
    ├── handlers.rs        # HTTP request handlers
    ├── client.rs          # Docker client utilities
    └── routes.rs          # Route registration
```

### Architectural Principles

1. **Separation of Concerns**: Each module handles a single domain (system, nginx, or docker)
2. **Layered Architecture**: Clear separation between models (data), handlers (logic), and routes (API)
3. **Dependency Injection**: Shared state (AppState) passed through actix-web's Data extractor
4. **Configuration Externalization**: All environment-specific values loaded from config
5. **Backward Compatibility**: All existing API endpoints preserved with identical behavior

## Components and Interfaces

### Configuration Module (config.rs)

**Purpose**: Centralize all configuration management and provide a single source of truth for application settings.

**Data Structures**:
```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub nginx: NginxConfig,
    pub docker: DockerConfig,
}

pub struct ServerConfig {
    pub bind_address: String,  // Default: "0.0.0.0:8080"
}

pub struct NginxConfig {
    pub sites_available_path: String,  // Default: "/etc/nginx/sites-available"
    pub sites_enabled_path: String,    // Default: "/etc/nginx/sites-enabled"
}

pub struct DockerConfig {
    pub socket_path: String,  // Default: "unix:///var/run/docker.sock"
}
```

**Interface**:
```rust
impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError>;
    pub fn with_defaults() -> Self;
}
```

### Error Module (error.rs)

**Purpose**: Provide consistent error handling across all modules.

**Data Structures**:
```rust
pub enum AppError {
    SystemError(String),
    NginxError(String),
    DockerError(String),
    ConfigError(String),
    NotFound(String),
    ValidationError(String),
}
```

**Interface**:
```rust
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse;
    fn status_code(&self) -> StatusCode;
}
```

### Utilities Module (utils.rs)

**Purpose**: Provide shared utility functions used across multiple modules.

**Interface**:
```rust
pub fn bytes_to_gb(bytes: u64) -> f64;
pub fn refresh_system(system: &mut System);
```

### System Module (system/)

**Purpose**: Handle all system monitoring functionality including CPU, memory, disk, network, and process information.

**Models (system/models.rs)**:
```rust
pub struct AppState {
    pub system: Arc<Mutex<System>>,
}

pub struct SystemInfo { /* existing fields */ }
pub struct CpuInfo { /* existing fields */ }
pub struct CpuUsage { /* existing fields */ }
pub struct MemoryInfo { /* existing fields */ }
pub struct DiskInfo { /* existing fields */ }
pub struct NetworkInfo { /* existing fields */ }
pub struct ProcessInfo { /* existing fields */ }
pub struct LoadAverage { /* existing fields */ }
pub struct HealthResponse { /* existing fields */ }
pub struct ApiInfo { /* existing fields */ }
pub struct ProcessQuery { /* existing fields */ }
pub struct KillResponse { /* existing fields */ }
```

**Handlers (system/handlers.rs)**:
```rust
pub async fn index() -> impl Responder;
pub async fn get_system_info(data: web::Data<AppState>) -> impl Responder;
pub async fn get_cpu_info(data: web::Data<AppState>) -> impl Responder;
pub async fn get_cpu_usage(data: web::Data<AppState>) -> impl Responder;
pub async fn get_memory_info(data: web::Data<AppState>) -> impl Responder;
pub async fn get_disks_info() -> impl Responder;
pub async fn get_network_info() -> impl Responder;
pub async fn get_processes(data: web::Data<AppState>, query: web::Query<ProcessQuery>) -> impl Responder;
pub async fn get_load_average() -> impl Responder;
pub async fn health_check() -> impl Responder;
pub async fn kill_process(data: web::Data<AppState>, pid: web::Path<u32>) -> impl Responder;
pub async fn dashboard() -> HttpResponse;
```

**Routes (system/routes.rs)**:
```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig);
```

### Nginx Module (nginx/)

**Purpose**: Handle all nginx proxy management functionality including CRUD operations and configuration generation.

**Models (nginx/models.rs)**:
```rust
pub struct NginxProxy { /* existing fields */ }
pub struct NginxResponse { /* existing fields */ }
pub struct FormatRequest { /* existing fields */ }
pub struct FormatResponse { /* existing fields */ }
```

**Config (nginx/config.rs)**:
```rust
pub fn format_nginx_config(config: &str) -> String;
pub fn validate_nginx_extra_config(config: &str) -> Result<String, String>;
pub fn generate_nginx_config(proxy: &NginxProxy) -> String;
```

**Handlers (nginx/handlers.rs)**:
```rust
pub async fn nginx_admin() -> HttpResponse;
pub async fn format_nginx_extra_config(req: web::Json<FormatRequest>) -> impl Responder;
pub async fn get_nginx_proxies() -> impl Responder;
pub async fn create_nginx_proxy(proxy: web::Json<NginxProxy>) -> impl Responder;
pub async fn delete_nginx_proxy(name: web::Path<String>) -> impl Responder;
pub async fn update_nginx_proxy(name: web::Path<String>, proxy: web::Json<NginxProxy>) -> impl Responder;
```

**Routes (nginx/routes.rs)**:
```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig);
```

### Docker Module (docker/)

**Purpose**: Handle all Docker management functionality including containers, images, volumes, and networks.

**Models (docker/models.rs)**:
```rust
pub struct DockerContainer { /* existing fields */ }
pub struct DockerImage { /* existing fields */ }
pub struct DockerVolume { /* existing fields */ }
pub struct DockerNetwork { /* existing fields */ }
pub struct DockerResponse { /* existing fields */ }
pub struct DockerLogsResponse { /* existing fields */ }
```

**Client (docker/client.rs)**:
```rust
pub async fn get_docker_client() -> Result<Docker, String>;
```

**Handlers (docker/handlers.rs)**:
```rust
pub async fn docker_manager() -> HttpResponse;
pub async fn list_containers() -> impl Responder;
pub async fn start_container(id: web::Path<String>) -> impl Responder;
pub async fn stop_container(id: web::Path<String>) -> impl Responder;
pub async fn restart_container(id: web::Path<String>) -> impl Responder;
pub async fn remove_container(id: web::Path<String>) -> impl Responder;
pub async fn get_container_logs(id: web::Path<String>) -> impl Responder;
pub async fn list_images() -> impl Responder;
pub async fn remove_image(id: web::Path<String>) -> impl Responder;
pub async fn list_volumes() -> impl Responder;
pub async fn remove_volume(name: web::Path<String>) -> impl Responder;
pub async fn list_networks() -> impl Responder;
pub async fn remove_network(id: web::Path<String>) -> impl Responder;
```

**Routes (docker/routes.rs)**:
```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig);
```

### Main Application (main.rs)

**Purpose**: Initialize the application, load configuration, and start the HTTP server.

**Structure**:
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Load configuration
    let config = AppConfig::from_env().unwrap_or_else(|_| AppConfig::with_defaults());
    
    // Initialize shared state
    let app_state = web::Data::new(AppState {
        system: Arc::new(Mutex::new(System::new_all())),
    });
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(system::routes::configure_routes)
            .configure(nginx::routes::configure_routes)
            .configure(docker::routes::configure_routes)
    })
    .bind(config.server.bind_address)?
    .run()
    .await
}
```

## Data Models

### System Module Models

All existing system-related structs will be moved to `system/models.rs`:
- `AppState`: Shared application state containing System instance
- `SystemInfo`: Overall system information
- `CpuInfo`: CPU specifications and details
- `CpuUsage`: Current CPU usage metrics
- `MemoryInfo`: Memory usage statistics
- `DiskInfo`: Disk usage information
- `NetworkInfo`: Network interface statistics
- `ProcessInfo`: Process details
- `LoadAverage`: System load averages
- `HealthResponse`: Health check response
- `ApiInfo`: API information
- `ProcessQuery`: Query parameters for process listing
- `KillResponse`: Response for process termination

### Nginx Module Models

All nginx-related structs will be moved to `nginx/models.rs`:
- `NginxProxy`: Nginx proxy configuration
- `NginxResponse`: Generic nginx operation response
- `FormatRequest`: Request for config formatting
- `FormatResponse`: Formatted config response

### Docker Module Models

All Docker-related structs will be moved to `docker/models.rs`:
- `DockerContainer`: Container information
- `DockerImage`: Image information
- `DockerVolume`: Volume information
- `DockerNetwork`: Network information
- `DockerResponse`: Generic Docker operation response
- `DockerLogsResponse`: Container logs response


## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Utility Function Purity

*For any* utility function in the utils module, calling it multiple times with the same input should produce the same output without side effects.

**Validates: Requirements 5.5**

This property ensures that utility functions like `bytes_to_gb` are pure functions that can be safely reused across modules without unexpected behavior. This is critical for maintainability and testability.

### Property 2: Dashboard Route Consistency

*For any* dashboard route (/, /nginx-admin, /docker-manager), accessing the route should return HTML content that matches the corresponding template file.

**Validates: Requirements 7.4**

This property ensures that after reorganizing HTML files into a templates directory, all dashboard routes still serve the correct content. This validates that the file loading mechanism works correctly.

### Property 3: Error Response Consistency

*For any* error returned by any handler in any module, the error response should use the AppError type and follow a consistent JSON format with the same field structure.

**Validates: Requirements 9.2, 9.3**

This property ensures that all modules use the common error handling infrastructure. Consistent error formats make the API easier to consume and debug. This validates that the error.rs module is properly integrated across all feature modules.

### Property 4: Error Backward Compatibility

*For any* error condition that existed in the original monolithic application, the refactored application should return the same error message and HTTP status code.

**Validates: Requirements 9.4, 9.5**

This property ensures that clients depending on specific error messages or status codes continue to work correctly after refactoring. This is critical for maintaining API contracts with existing clients.

### Property 5: Complete API Backward Compatibility

*For any* API endpoint with any valid request, the refactored application should return a response identical to the original monolithic application (same status code, same response body structure, same data).

**Validates: Requirements 2.4, 3.5, 4.5, 10.2, 10.4**

This is the most critical property for the refactoring. It ensures that despite the internal reorganization, all external behavior remains unchanged. This validates that:
- System monitoring endpoints return correct data structures
- Nginx management endpoints use the separated handlers correctly
- Docker management endpoints use the separated handlers correctly
- All request and response formats are preserved
- All functionality is preserved

This property subsumes several individual requirements because if the entire API behaves identically, then by definition each subsystem is working correctly.

## Error Handling

### Error Type Hierarchy

The application will use a centralized error type defined in `error.rs`:

```rust
#[derive(Debug)]
pub enum AppError {
    SystemError(String),
    NginxError(String),
    DockerError(String),
    ConfigError(String),
    NotFound(String),
    ValidationError(String),
}
```

### Error Response Format

All errors will be returned as JSON with a consistent structure:

```json
{
    "error": "Error type",
    "message": "Detailed error message"
}
```

### Error Handling Strategy

1. **Module-Specific Errors**: Each module converts its internal errors to AppError variants
2. **HTTP Status Mapping**: AppError implements ResponseError to map to appropriate HTTP status codes
3. **Error Logging**: All errors are logged with context before being returned to clients
4. **Backward Compatibility**: Existing error messages and status codes are preserved

### Error Propagation

```rust
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
            // ... other variants
        }
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

## Testing Strategy

### Dual Testing Approach

The refactoring will be validated using both unit tests and property-based tests:

- **Unit tests**: Verify specific examples, edge cases, and structural requirements
- **Property tests**: Verify universal properties across all inputs

Both testing approaches are complementary and necessary for comprehensive coverage. Unit tests catch concrete bugs and verify specific scenarios, while property tests verify general correctness across a wide range of inputs.

### Unit Testing Focus

Unit tests should focus on:

1. **Structural Validation**: Verify that expected files and modules exist
   - Check that all module files are created (models.rs, handlers.rs, routes.rs)
   - Verify that lib.rs properly exposes modules
   - Confirm that main.rs is minimal (< 100 lines)

2. **Configuration Loading**: Test specific configuration scenarios
   - Test loading from environment variables
   - Test default value fallback
   - Test invalid configuration handling

3. **Module Integration**: Test that modules are properly wired together
   - Test that route registration functions are called
   - Test that handlers can access shared state
   - Test that error types are properly converted

4. **Edge Cases**: Test boundary conditions
   - Empty configuration values
   - Missing HTML template files
   - Invalid nginx configurations
   - Docker client connection failures

### Property-Based Testing Focus

Property tests should focus on the correctness properties defined above. We will use the `proptest` crate for Rust property-based testing.

**Configuration**:
- Minimum 100 iterations per property test
- Each test tagged with feature name and property number
- Tag format: `// Feature: application-refactoring, Property N: [property text]`

**Property Test Implementation**:

1. **Property 1: Utility Function Purity**
   - Generate random byte values
   - Call bytes_to_gb multiple times with same input
   - Verify identical outputs

2. **Property 2: Dashboard Route Consistency**
   - Test all dashboard routes
   - Verify HTML content matches template files
   - Verify content-type headers

3. **Property 3: Error Response Consistency**
   - Generate various error conditions across all modules
   - Verify all errors use AppError type
   - Verify consistent JSON structure

4. **Property 4: Error Backward Compatibility**
   - Compare error responses between original and refactored versions
   - Verify same status codes
   - Verify same error messages

5. **Property 5: Complete API Backward Compatibility**
   - Generate random valid requests for all endpoints
   - Compare responses between original and refactored versions
   - Verify identical behavior

### Testing Implementation Notes

- Property tests will require running both the original and refactored versions for comparison
- Consider using snapshot testing for backward compatibility validation
- Integration tests should verify end-to-end flows through the refactored modules
- Performance tests should verify that refactoring doesn't introduce performance regressions

### Test Organization

```
tests/
├── unit/
│   ├── config_tests.rs
│   ├── structure_tests.rs
│   └── module_tests.rs
├── integration/
│   ├── system_api_tests.rs
│   ├── nginx_api_tests.rs
│   └── docker_api_tests.rs
└── property/
    ├── utility_properties.rs
    ├── dashboard_properties.rs
    ├── error_properties.rs
    └── compatibility_properties.rs
```

## Migration Strategy

### Phase 1: Create Module Structure
1. Create directory structure (system/, nginx/, docker/)
2. Create empty module files (mod.rs, models.rs, handlers.rs, routes.rs)
3. Create lib.rs with module declarations
4. Verify compilation with empty modules

### Phase 2: Extract Models
1. Move data structures to respective models.rs files
2. Add necessary imports and derives
3. Update lib.rs to export models
4. Verify compilation

### Phase 3: Extract Utilities
1. Create utils.rs and error.rs
2. Move utility functions to utils.rs
3. Create AppError enum in error.rs
4. Update imports across codebase

### Phase 4: Extract Handlers
1. Move handler functions to respective handlers.rs files
2. Update imports to use new module paths
3. Convert error handling to use AppError
4. Verify compilation

### Phase 5: Create Route Registration
1. Create routes.rs for each module
2. Implement configure_routes functions
3. Update main.rs to call route registration functions
4. Verify all routes are accessible

### Phase 6: Configuration Management
1. Create config.rs with AppConfig struct
2. Implement configuration loading from environment
3. Replace hard-coded values with config references
4. Test configuration loading

### Phase 7: HTML Template Organization
1. Create templates/ directory
2. Move HTML files to templates/
3. Update handlers to load from new location
4. Verify dashboards still work

### Phase 8: Testing and Validation
1. Run all unit tests
2. Run all property tests
3. Perform manual API testing
4. Verify backward compatibility
5. Performance testing

### Rollback Strategy

If issues are discovered:
1. Keep original main.rs as main.rs.backup
2. Use feature flags to switch between old and new implementations
3. Maintain both versions until refactored version is fully validated
4. Use git branches for safe experimentation

## Performance Considerations

The refactoring should not introduce performance regressions:

1. **Module Loading**: Rust's module system has zero runtime overhead
2. **Function Calls**: Cross-module function calls are inlined by the compiler
3. **Error Handling**: AppError enum is stack-allocated, no heap overhead
4. **Configuration**: Config loaded once at startup, no runtime impact
5. **Route Registration**: Routes registered at startup, no request-time overhead

The refactored application should have identical performance characteristics to the original monolithic version.

## Security Considerations

The refactoring maintains all existing security properties:

1. **Input Validation**: All existing validation logic preserved
2. **File System Access**: Nginx config file operations remain restricted
3. **Docker Socket Access**: Docker client security unchanged
4. **Process Management**: Process kill operations maintain same restrictions
5. **Configuration**: No sensitive data in configuration files (use environment variables)

## Documentation Requirements

After refactoring:

1. Update README.md with new module structure
2. Document configuration options and environment variables
3. Add module-level documentation comments
4. Document the route registration pattern
5. Provide migration guide for contributors
6. Update API documentation if needed

## Future Extensibility

The refactored structure enables future improvements:

1. **New Features**: Easy to add new modules following established pattern
2. **Testing**: Easier to test individual modules in isolation
3. **Configuration**: Easy to add new configuration options
4. **Error Handling**: Easy to add new error types
5. **Middleware**: Easier to add module-specific middleware
6. **API Versioning**: Structure supports versioned API modules

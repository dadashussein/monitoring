# Requirements Document

## Introduction

This document specifies the requirements for refactoring the Ubuntu Resource Monitor application from a monolithic single-file structure into a well-organized, modular Rust application. The refactoring aims to improve maintainability, testability, and code organization while preserving all existing functionality.

## Glossary

- **Application**: The Ubuntu Resource Monitor REST API application
- **Module**: A separate Rust file containing related functionality
- **Handler**: An actix-web route handler function that processes HTTP requests
- **Model**: A data structure (struct) representing domain entities
- **System_Monitor**: The subsystem responsible for collecting and reporting system metrics
- **Nginx_Manager**: The subsystem responsible for managing nginx proxy configurations
- **Docker_Manager**: The subsystem responsible for managing Docker containers, images, volumes, and networks
- **Configuration**: Application settings loaded from environment variables or configuration files
- **Hard_Coded_Value**: A literal value embedded directly in source code

## Requirements

### Requirement 1: Module Structure Organization

**User Story:** As a developer, I want the codebase organized into logical modules, so that I can easily locate and modify specific functionality.

#### Acceptance Criteria

1. THE Application SHALL organize code into separate module files following Rust module conventions
2. WHEN the application starts, THE Application SHALL load all modules from the src directory structure
3. THE Application SHALL maintain a module hierarchy with models, handlers, and utilities separated
4. THE Application SHALL use a lib.rs file to expose public module interfaces
5. THE Application SHALL keep main.rs minimal, containing only application initialization and server startup

### Requirement 2: System Monitoring Module Separation

**User Story:** As a developer, I want system monitoring functionality isolated in its own module, so that system metrics collection is independent from other features.

#### Acceptance Criteria

1. THE System_Monitor SHALL contain all system information data structures in a dedicated models file
2. THE System_Monitor SHALL contain all system monitoring HTTP handlers in a dedicated handlers file
3. THE System_Monitor SHALL expose functions for CPU, memory, disk, network, and process information
4. WHEN system information is requested, THE System_Monitor SHALL return data using the defined model structures
5. THE System_Monitor SHALL maintain the AppState structure for shared system state

### Requirement 3: Nginx Management Module Separation

**User Story:** As a developer, I want nginx proxy management functionality isolated in its own module, so that proxy configuration logic is independent from other features.

#### Acceptance Criteria

1. THE Nginx_Manager SHALL contain all nginx-related data structures in a dedicated models file
2. THE Nginx_Manager SHALL contain all nginx HTTP handlers in a dedicated handlers file
3. THE Nginx_Manager SHALL contain nginx configuration generation logic in a utilities file
4. THE Nginx_Manager SHALL contain nginx configuration validation logic in a utilities file
5. WHEN nginx proxy operations are requested, THE Nginx_Manager SHALL use the separated handler functions

### Requirement 4: Docker Management Module Separation

**User Story:** As a developer, I want Docker management functionality isolated in its own module, so that container orchestration logic is independent from other features.

#### Acceptance Criteria

1. THE Docker_Manager SHALL contain all Docker-related data structures in a dedicated models file
2. THE Docker_Manager SHALL contain all Docker HTTP handlers in a dedicated handlers file
3. THE Docker_Manager SHALL contain Docker client initialization logic in a utilities file
4. THE Docker_Manager SHALL expose functions for container, image, volume, and network management
5. WHEN Docker operations are requested, THE Docker_Manager SHALL use the separated handler functions

### Requirement 5: Shared Utilities Extraction

**User Story:** As a developer, I want common utility functions extracted into a shared module, so that code duplication is minimized.

#### Acceptance Criteria

1. THE Application SHALL contain a utilities module for shared helper functions
2. THE Application SHALL place conversion functions (such as bytes_to_gb) in the utilities module
3. THE Application SHALL place system refresh logic in the utilities module
4. WHEN any module needs utility functions, THE Application SHALL import them from the utilities module
5. THE Application SHALL ensure utility functions are stateless and reusable

### Requirement 6: Configuration Management

**User Story:** As a developer, I want hard-coded values replaced with configuration, so that the application can be deployed in different environments without code changes.

#### Acceptance Criteria

1. THE Application SHALL load configuration from environment variables or a configuration file
2. THE Application SHALL make the server bind address configurable (currently hard-coded to 0.0.0.0:8080)
3. THE Application SHALL make nginx configuration paths configurable (currently hard-coded to /etc/nginx/sites-available and /etc/nginx/sites-enabled)
4. THE Application SHALL make Docker socket path configurable (currently uses default)
5. THE Application SHALL provide sensible default values when configuration is not specified

### Requirement 7: HTML Dashboard Separation

**User Story:** As a developer, I want HTML dashboard files organized separately from Rust code, so that frontend assets are clearly distinguished from backend logic.

#### Acceptance Criteria

1. THE Application SHALL maintain HTML files in a dedicated templates or static directory
2. THE Application SHALL load HTML content from files at runtime or embed them at compile time
3. THE Application SHALL serve dashboard.html, nginx_admin.html, and docker_manager.html from the organized location
4. WHEN a dashboard route is accessed, THE Application SHALL return the corresponding HTML content
5. THE Application SHALL maintain all existing dashboard functionality after reorganization

### Requirement 8: Route Registration Organization

**User Story:** As a developer, I want route registration organized by feature module, so that API endpoints are clearly associated with their functionality.

#### Acceptance Criteria

1. THE Application SHALL define route registration functions within each feature module
2. THE System_Monitor SHALL provide a function to register all system monitoring routes
3. THE Nginx_Manager SHALL provide a function to register all nginx management routes
4. THE Docker_Manager SHALL provide a function to register all Docker management routes
5. WHEN the application starts, THE Application SHALL call each module's route registration function

### Requirement 9: Error Handling Consistency

**User Story:** As a developer, I want consistent error handling across all modules, so that errors are reported uniformly to API clients.

#### Acceptance Criteria

1. THE Application SHALL define common error types in a shared module
2. THE Application SHALL use consistent error response formats across all handlers
3. WHEN an error occurs in any module, THE Application SHALL return errors using the common error types
4. THE Application SHALL preserve all existing error messages and HTTP status codes
5. THE Application SHALL log errors consistently across all modules

### Requirement 10: Backward Compatibility

**User Story:** As a user, I want all existing API endpoints to continue working, so that clients are not broken by the refactoring.

#### Acceptance Criteria

1. THE Application SHALL maintain all existing API endpoint paths
2. THE Application SHALL maintain all existing request and response formats
3. THE Application SHALL maintain all existing HTTP methods for each endpoint
4. WHEN any existing API endpoint is called, THE Application SHALL return responses identical to the pre-refactoring version
5. THE Application SHALL preserve all existing functionality including system monitoring, nginx management, and Docker management

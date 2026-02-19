# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release
- System monitoring (CPU, memory, disk, network, processes)
- Nginx proxy manager
- Docker container manager
- Real-time web dashboard
- Systemd service installation
- Pre-built binaries for x86_64, ARM64, and ARMv7
- One-command installation script
- Configuration via environment variables

### Changed
- Refactored from monolithic structure to modular architecture
- Improved code organization and maintainability

### Security
- All operations require root privileges
- Systemd service runs as root for system access

## [1.0.0] - YYYY-MM-DD

### Added
- Initial public release
- System resource monitoring
- Process management
- Nginx reverse proxy configuration
- Docker container management
- Web-based dashboard
- RESTful API
- Systemd service integration

---

## How to Update This File

When preparing a new release:

1. Move items from `[Unreleased]` to a new version section
2. Add the release date
3. Create a new `[Unreleased]` section for future changes
4. Follow these categories:
   - **Added** - New features
   - **Changed** - Changes in existing functionality
   - **Deprecated** - Soon-to-be removed features
   - **Removed** - Removed features
   - **Fixed** - Bug fixes
   - **Security** - Security fixes

Example:
```markdown
## [1.1.0] - 2024-02-15

### Added
- New feature X
- Support for Y

### Fixed
- Bug in Z component
```

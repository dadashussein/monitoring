# Contributing to Ubuntu Resource API

Thank you for your interest in contributing! This document provides guidelines for contributing to this project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/ubuntu-resource-api.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites
- Rust 1.70 or higher
- Docker (for testing Docker management features)
- Nginx (for testing proxy management features)

### Building the Project

```bash
# Build in development mode
cargo build

# Build in release mode
cargo build --release

# Run with logging
RUST_LOG=info cargo run
```

### Using Docker

```bash
# Build Docker image
make build

# Run in Docker
make run

# Stop container
make stop
```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy for linting: `cargo clippy`
- Ensure all tests pass before submitting

## Making Changes

1. Make your changes in your feature branch
2. Add tests for new functionality
3. Update documentation if needed
4. Run `cargo fmt` and `cargo clippy`
5. Commit with clear, descriptive messages

### Commit Message Format

```
type: brief description

Detailed explanation if needed

Fixes #issue_number
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Submitting Changes

1. Push your changes to your fork
2. Create a Pull Request to the main repository
3. Fill out the PR template completely
4. Wait for review and address any feedback

## Testing

- Write unit tests for new functions
- Test Docker operations manually if adding Docker features
- Test Nginx proxy configurations if modifying proxy features
- Verify system monitoring endpoints work correctly

## Reporting Issues

- Use the issue template
- Provide clear reproduction steps
- Include system information (OS, Rust version, Docker version)
- Add relevant logs or error messages

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## Questions?

Feel free to open an issue for questions or discussions about contributing.

Thank you for contributing! 🚀

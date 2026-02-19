# Configuration
BINARY_NAME := ubuntu_resource_api

.PHONY: help install uninstall build run clean release test

# Default target
help:
	@echo "Ubuntu Resource Monitor - Available Commands:"
	@echo ""
	@echo "Installation:"
	@echo "  make install          - Install as systemd service (requires sudo)"
	@echo "  make uninstall        - Remove systemd service (requires sudo)"
	@echo ""
	@echo "Development:"
	@echo "  make build            - Build release binary"
	@echo "  make run              - Run the application"
	@echo "  make test             - Run tests"
	@echo "  make clean            - Clean build artifacts"
	@echo ""
	@echo "Release:"
	@echo "  make release          - Create GitHub release with binaries"
	@echo ""

# Install as systemd service
install:
	@echo "🚀 Installing Ubuntu Resource Monitor..."
	@chmod +x install.sh
	@sudo ./install.sh

# Uninstall systemd service
uninstall:
	@echo "🗑️  Uninstalling Ubuntu Resource Monitor..."
	@chmod +x uninstall.sh
	@sudo ./uninstall.sh

# Build release binary
build:
	@echo "🔨 Building release binary..."
	cargo build --release

# Run the application
run: build
	@echo "🚀 Starting Ubuntu Resource Monitor..."
	./target/release/$(BINARY_NAME)

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Create GitHub release
release:
	@echo "🏷️  Creating GitHub release..."
	@echo "Make sure you have created a tag first:"
	@echo "  git tag v1.0.0"
	@echo "  git push origin v1.0.0"
	@echo ""
	@echo "Then GitHub Actions will automatically build and release binaries."
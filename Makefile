.PHONY: build run clean help

# Configuration
ADDRESS ?= 10.0.0.1
PORT ?= 3012

# Build binary using Docker (no Rust installation needed)
build:
	@echo "🔨 Building binary with Docker..."
	docker build --network=host -t ubuntu-resource-api-builder .
	@mkdir -p target/release
	docker run --rm -v $(PWD)/target/release:/output ubuntu-resource-api-builder sh -c "cp /app/target/release/ubuntu_resource_api /output/ 2>/dev/null || echo '⚠️  Binary not found in container, rebuilding...'"
	@if [ ! -f ./target/release/ubuntu_resource_api ]; then \
		echo "🔄 Forcing rebuild without cache..."; \
		docker build --no-cache --network=host -t ubuntu-resource-api-builder .; \
		docker run --rm -v $(PWD)/target/release:/output ubuntu-resource-api-builder sh -c "cp /app/target/release/ubuntu_resource_api /output/"; \
	fi
	chmod +x ./target/release/ubuntu_resource_api
	@echo "✅ Binary created: ./target/release/ubuntu_resource_api"

# Run the binary locally
run: build
	@echo "🚀 Starting server on $(ADDRESS):$(PORT)..."
	./target/release/ubuntu_resource_api $(ADDRESS) $(PORT)

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	rm -rf target/
	docker rmi ubuntu-resource-api-builder 2>/dev/null || true
	@echo "✅ Cleaned"

# Show help
help:
	@echo "Available commands:"
	@echo "  make build  - Build binary using Docker (no Rust needed)"
	@echo "  make run    - Build and run the server"
	@echo "  make clean  - Remove build artifacts"
	@echo "  make help   - Show this help"

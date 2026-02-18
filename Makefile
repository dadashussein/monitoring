.PHONY: build run docker-build docker-run docker-stop docker-logs clean help

# Configuration
ADDRESS ?= 10.0.0.1
PORT ?= 3012
CONTAINER_NAME ?= ubuntu-resource-api

# Build binary using Docker (no Rust installation needed)
build:
	@echo "🔨 Building binary with Docker..."
	docker build --network=host -t ubuntu-resource-api-builder .
	@mkdir -p target/release
	docker run --rm -v $(CURDIR)/target/release:/output ubuntu-resource-api-builder sh -c "cp /ubuntu_resource_api /output/ubuntu_resource_api"
	chmod +x ./target/release/ubuntu_resource_api
	@echo "✅ Binary created: ./target/release/ubuntu_resource_api"

# Build runtime Docker image
docker-build: build
	@echo "🐳 Building runtime Docker image..."
	docker build -f Dockerfile.runtime -t ubuntu-resource-api .
	@echo "✅ Docker image created: ubuntu-resource-api"

# Run as Docker container
docker-run: docker-build
	@echo "🚀 Starting Docker container..."
	@docker stop $(CONTAINER_NAME) 2>/dev/null || true
	@docker rm $(CONTAINER_NAME) 2>/dev/null || true
	docker run -d \
		--name $(CONTAINER_NAME) \
		--restart unless-stopped \
		-p $(ADDRESS):$(PORT):3012 \
		-v /etc/nginx/sites-available:/etc/nginx/sites-available \
		-v /etc/nginx/sites-enabled:/etc/nginx/sites-enabled \
		-v /var/run:/var/run \
		-v /var/run/docker.sock:/var/run/docker.sock \
		--privileged \
		ubuntu-resource-api
	@echo "✅ Container started: $(CONTAINER_NAME)"
	@echo "📊 Dashboard: http://$(ADDRESS):$(PORT)/dashboard"
	@echo "🔄 Nginx Manager: http://$(ADDRESS):$(PORT)/nginx"
	@echo "🐳 Docker Manager: http://$(ADDRESS):$(PORT)/docker"
	@echo "🔍 Logs: make docker-logs"

# Stop Docker container
docker-stop:
	@echo "🛑 Stopping container..."
	docker stop $(CONTAINER_NAME)
	docker rm $(CONTAINER_NAME)
	@echo "✅ Container stopped"

# Show container logs
docker-logs:
	docker logs -f $(CONTAINER_NAME)

# Run the binary locally (not in Docker)
run: build
	@echo "🚀 Starting server on $(ADDRESS):$(PORT)..."
	./target/release/ubuntu_resource_api $(ADDRESS) $(PORT)

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	rm -rf target/
	docker rmi ubuntu-resource-api-builder 2>/dev/null || true
	docker rmi ubuntu-resource-api 2>/dev/null || true
	@docker stop $(CONTAINER_NAME) 2>/dev/null || true
	@docker rm $(CONTAINER_NAME) 2>/dev/null || true
	@echo "✅ Cleaned"

# Show help
help:
	@echo "Available commands:"
	@echo "  make build        - Build binary using Docker (no Rust needed)"
	@echo "  make docker-build - Build runtime Docker image"
	@echo "  make docker-run   - Build and run as Docker container (recommended)"
	@echo "  make docker-stop  - Stop and remove Docker container"
	@echo "  make docker-logs  - Show container logs"
	@echo "  make run          - Build and run locally (not in Docker)"
	@echo "  make clean        - Remove all build artifacts and containers"
	@echo "  make help         - Show this help"

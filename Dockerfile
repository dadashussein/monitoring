# Build binary using Docker
FROM rust:1.83-slim

WORKDIR /app

# Copy project files
COPY Cargo.toml ./
COPY src ./src

# Build release binary (will generate new Cargo.lock)
RUN cargo build --release

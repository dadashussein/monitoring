# Build binary using Docker
FROM rust:1.88-slim

WORKDIR /app

# Configure cargo for better network handling
ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_TIMEOUT=300

# Set up cargo config with sparse registry (faster and more reliable)
RUN mkdir -p /usr/local/cargo && \
    echo '[registries.crates-io]' > /usr/local/cargo/config.toml && \
    echo 'protocol = "sparse"' >> /usr/local/cargo/config.toml

# Copy project files
COPY Cargo.toml ./
COPY src ./src

# Build release binary (will generate new Cargo.lock)
RUN cargo build --release

# Copy binary to a known location for easy extraction
RUN cp /app/target/release/ubuntu_resource_api /ubuntu_resource_api

CMD ["cp", "/ubuntu_resource_api", "/output/ubuntu_resource_api"]

#!/bin/bash
set -e

echo "🔧 Building static binary for ubuntu-resource-api..."

# Install musl target if not present
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

# Build static binary
RUSTFLAGS='-C target-feature=+crt-static' \
    cargo build --release --target x86_64-unknown-linux-musl

echo ""
echo "✅ Static binary built successfully!"
echo "📦 Location: target/x86_64-unknown-linux-musl/release/ubuntu_resource_api"
echo ""
echo "🚀 To run on another machine (no Rust required):"
echo "   scp target/x86_64-unknown-linux-musl/release/ubuntu_resource_api user@host:~/"
echo "   ssh user@host ./ubuntu_resource_api"
echo ""
echo "🌐 Then open: http://<host-ip>:8080/dashboard"

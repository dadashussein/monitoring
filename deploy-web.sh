#!/bin/bash
# Deploy production website with downloadable binary

set -e

WEB_DIR="./web"
DIST_DIR="./dist"
BINARY="./target/release/ubuntu_resource_api"

echo "🚀 Building production release..."
cargo build --release

echo "📦 Preparing distribution..."
mkdir -p "$DIST_DIR"
cp "$BINARY" "$DIST_DIR/ubuntu_resource_api-linux-x86_64"
cp "$WEB_DIR/index.html" "$DIST_DIR/"
cp "$WEB_DIR/install" "$DIST_DIR/"

echo "✅ Production files ready in $DIST_DIR/"
echo ""
echo "📁 Files to upload to your server:"
ls -la "$DIST_DIR/"
echo ""
echo "🌐 Upload to your web server:"
echo "  scp -r $DIST_DIR/* user@your-server:/var/www/html/"
echo ""
echo "🔗 Your install command will be:"
echo "  curl -sSL https://your-domain.com/install | sudo bash"

#!/bin/bash
# Deploy script for Ubuntu Resource API
# Usage: ./deploy.sh user@remote-host

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 user@remote-host [remote-port]"
    echo "Example: $0 ubuntu@192.168.1.100"
    exit 1
fi

REMOTE_HOST="$1"
REMOTE_PORT="${2:-8080}"
BINARY="target/release/ubuntu_resource_api"

echo "🚀 Deploying Ubuntu Resource API to $REMOTE_HOST..."

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found. Building..."
    cargo build --release
fi

# Copy binary
echo "📦 Copying binary..."
scp "$BINARY" "$REMOTE_HOST:~/ubuntu_resource_api"

# Make executable and run
echo "🎯 Starting service on $REMOTE_HOST..."
ssh "$REMOTE_HOST" "chmod +x ~/ubuntu_resource_api && pkill -f ubuntu_resource_api 2>/dev/null; nohup ~/ubuntu_resource_api > /dev/null 2>&1 &"

# Wait a moment and check
echo "⏳ Waiting for service to start..."
sleep 2

if ssh "$REMOTE_HOST" "curl -s http://localhost:$REMOTE_PORT/health" > /dev/null 2>&1; then
    echo ""
    echo "✅ Deployment successful!"
    echo ""
    echo "🌐 Dashboard: http://$REMOTE_HOST:$REMOTE_PORT/dashboard"
    echo "📡 API:       http://$REMOTE_HOST:$REMOTE_PORT/"
    echo ""
    echo "To stop the service:"
    echo "  ssh $REMOTE_HOST 'pkill -f ubuntu_resource_api'"
else
    echo ""
    echo "⚠️  Service may not have started properly."
    echo "Check logs: ssh $REMOTE_HOST '~/ubuntu_resource_api'"
fi

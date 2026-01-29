# Production Deployment Guide

## 🚀 One-Line Install Setup

Your users will install with:
```bash
curl -sSL https://your-domain.com/install | sudo bash
```

## 📋 Setup Steps

### 1. Prepare Your Domain

Point your domain (e.g., `monitor.yourdomain.com`) to your server IP.

### 2. Build & Prepare

```bash
# Build the release binary
cargo build --release

# Prepare distribution
./deploy-web.sh
```

### 3. Upload to Web Server

```bash
# Option A: Using SCP
scp -r dist/* root@your-server:/var/www/html/

# Option B: Using rsync
rsync -avz dist/ root@your-server:/var/www/html/
```

### 4. Configure Web Server

#### Nginx (Recommended)

```bash
sudo apt install nginx
sudo tee /etc/nginx/sites-available/resource-api << 'EOF'
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/html;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
    
    location ~ ^/(install|ubuntu_resource_api-linux-) {
        add_header Content-Type text/plain;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/resource-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

#### Using Caddy (Auto HTTPS)

```bash
sudo apt install caddy
sudo tee /etc/caddy/Caddyfile << 'EOF'
your-domain.com {
    root * /var/www/html
    file_server
}
EOF
sudo systemctl restart caddy
```

### 5. Test Installation

From a fresh Linux machine:
```bash
curl -sSL https://your-domain.com/install | sudo bash
```

Then open: `http://server-ip:8080/dashboard`

---

## 🔧 Files Structure

```
dist/
├── index.html           # Landing page
├── install              # Install script (executable)
└── ubuntu_resource_api-linux-x86_64  # Binary
```

## 🧪 Test Locally

```bash
# Build everything
./deploy-web.sh

# Serve locally
python3 serve-web.py

# Open http://localhost:8000
```

## 📝 Update install.sh

Edit `install.sh` and update:
```bash
REPO_URL="https://github.com/YOUR_USERNAME/ubuntu-resource-api"
# Change DOWNLOAD_URL to your domain:
DOWNLOAD_URL="https://your-domain.com/download/${BINARY_NAME}-linux-${ARCH}"
```

## 🔄 Update Release

```bash
# After making changes
cargo build --release
./deploy-web.sh
scp -r dist/* your-server:/var/www/html/
```

## 📊 GitHub Actions (Auto Release)

Create `.github/workflows/release.yml`:

```yaml
name: Release
on:
  push:
    tags:
      - 'v*'
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-unknown-linux-gnu
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/ubuntu_resource_api
```

UBUNTU RESOURCE API - PORTABLE BINARY
=====================================

This is a standalone binary that runs on any Linux x86_64 system.
No Rust installation required!

QUICK START
-----------
1. Copy ubuntu_resource_api to your target machine:
   scp ubuntu_resource_api user@remote-server:~/

2. Make it executable:
   chmod +x ubuntu_resource_api

3. Run it:
   ./ubuntu_resource_api

4. Open in browser:
   http://<server-ip>:8080/dashboard

REQUIREMENTS
------------
- Linux x86_64 (64-bit)
- glibc 2.31+ (Ubuntu 20.04+, Debian 11+, CentOS 8+, etc.)

The binary only uses standard system libraries (libc, libm, libgcc).

SYSTEMD SERVICE (Optional)
--------------------------
To run as a service, create /etc/systemd/system/ubuntu-resource-api.service:

[Unit]
Description=Ubuntu Resource API
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/ubuntu_resource_api
Restart=always
User=ubuntu

[Install]
WantedBy=multi-user.target

Then:
  sudo cp ubuntu_resource_api /usr/local/bin/
  sudo systemctl daemon-reload
  sudo systemctl enable ubuntu-resource-api
  sudo systemctl start ubuntu-resource-api

API ENDPOINTS
-------------
GET /           - API info
GET /dashboard  - Web dashboard
GET /api/system - System information
GET /api/cpu    - CPU information
GET /api/memory - Memory usage
GET /api/disks  - Disk usage
GET /api/network- Network interfaces
GET /api/processes?limit=N - Top processes
GET /api/load   - Load average
GET /health     - Health check

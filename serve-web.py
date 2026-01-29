#!/usr/bin/env python3
"""
Simple HTTP server for testing the production website locally.
Usage: python3 serve-web.py
Then open: http://localhost:8000
"""

import http.server
import socketserver
import os

PORT = 8000
SERVE_DIR = "dist"

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=SERVE_DIR, **kwargs)

os.makedirs(SERVE_DIR, exist_ok=True)

with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"🌐 Serving at http://localhost:{PORT}")
    print(f"📁 Serving directory: {SERVE_DIR}/")
    print("Press Ctrl+C to stop")
    httpd.serve_forever()

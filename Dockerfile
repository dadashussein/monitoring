# --- Stage 1: Builder (Rust Derleme Aşaması) ---
FROM rust:bookworm as builder

WORKDIR /app
COPY . .

# Release modunda derle
RUN cargo build --release

# --- Stage 2: Runtime (Çalışma Aşaması) ---
FROM debian:bookworm-slim

# Gerekli araçları kur (util-linux, nsenter içerir)
RUN apt-get update && \
    apt-get install -y nginx util-linux && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled

WORKDIR /app

# --- MAGIC FIX: Systemctl Wrapper ---
# Rust uygulaman "systemctl" çağırdığında, aslında host makinede çalışması için
# bir sarmalayıcı (wrapper) script yazıyoruz.
RUN echo '#!/bin/bash\nnsenter --target 1 --mount --uts --ipc --net --pid systemctl "$@"' > /usr/local/bin/systemctl && \
    chmod +x /usr/local/bin/systemctl

# Derlenen binary'yi kopyala
COPY --from=builder /app/target/release/ubuntu_resource_api /app/ubuntu_resource_api
RUN chmod +x /app/ubuntu_resource_api

# Portu dışarı aç
EXPOSE 3012

# Log seviyesi
ENV RUST_LOG=info

# Root olarak çalıştır
USER root

ENV APP_HOST=0.0.0.0
ENV APP_PORT=3012

# Wrapper ve Binary'yi değişkenlerle başlat
# DİKKAT: Burada sh -c kullanıyoruz ki $APP_HOST ve $APP_PORT okunabilsin.
CMD ["sh", "-c", "/app/ubuntu_resource_api ${APP_HOST} ${APP_PORT}"]
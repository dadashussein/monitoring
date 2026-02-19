# Configuration
IMAGE_NAME := dadashussein/resource-manager
TAG := latest
PLATFORM := linux/amd64

# ... (Mevcut clean, help vs. komutların kalabilir) ...

# Tek komutla hem build et hem de Docker Hub'a gönder
publish:
	@echo "🔐 Docker Hub'a giriş yapılıyor..."
	docker login
	@echo "🐳 Docker imajı build ediliyor..."
	docker build --platform $(PLATFORM) -t $(IMAGE_NAME):$(TAG) .
	@echo "🚀 Docker Hub'a pushlanıyor..."
	docker push $(IMAGE_NAME):$(TAG)
	@echo "✅ İşlem tamam! Müşterilerin artık şu komutu kullanabilir:"
	@echo "docker pull $(IMAGE_NAME):$(TAG)"
.PHONY: build-core build-proxy build-queue build-all deploy

build-core:
	docker build -t nilferreira/anibalmf1-rinha-2025-core ./core

build-proxy:
	docker build -t nilferreira/anibalmf1-rinha-2025-proxy ./proxy


build-all: build-core build-proxy build-queue

deploy:
	docker push nilferreira/anibalmf1-rinha-2025-core
	docker push nilferreira/anibalmf1-rinha-2025-proxy

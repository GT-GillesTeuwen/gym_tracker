# Variables
IMAGE_NAME=gym_tracker_api
PLATFORM=linux/amd64
REGISTRY=192.168.8.150:5000
IMAGE_TAG=$(REGISTRY)/$(IMAGE_NAME):latest
BUILDKIT_CONFIG=./buildkitd.toml

# Ensure buildx is set up
.PHONY: setup-buildx
setup-buildx:
	docker buildx rm multiarch-builder || true
	docker buildx create --use --name multiarch-builder --driver docker-container --config $(BUILDKIT_CONFIG)
	docker buildx inspect --bootstrap

# Build for the specified platform
.PHONY: build
build: setup-buildx
	docker buildx build \
		--platform $(PLATFORM) \
		--cache-to=type=registry,ref=$(IMAGE_TAG)-cache,mode=max \
		--cache-from=type=registry,ref=$(IMAGE_TAG)-cache \
		-t $(IMAGE_TAG) \
		--push .


.PHONY: build_no_cache
build_no_cache: setup-buildx
	docker buildx build --no-cache --platform $(PLATFORM) -t $(IMAGE_TAG) --push .

# Optional: clean up the builder instance
.PHONY: cleanup
cleanup:
	docker buildx rm multiarch-builder || true

list_images:
	curl -X GET https://${REGISTRY}/v2/_catalog
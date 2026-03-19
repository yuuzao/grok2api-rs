IMAGE ?= yuuzao/grok2api-rs
TAG ?= latest
PLATFORMS ?= linux/amd64,linux/arm64
VCS_REF ?= $(shell git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)
TAG_ARGS := -t $(IMAGE):$(TAG)

ifneq ($(TAG),latest)
TAG_ARGS += -t $(IMAGE):latest
endif

.PHONY: build push

build:
	docker build \
		--build-arg VCS_REF=$(VCS_REF) \
		$(TAG_ARGS) \
		.

push:
	docker buildx build \
		--platform $(PLATFORMS) \
		--build-arg VCS_REF=$(VCS_REF) \
		$(TAG_ARGS) \
		--push \
		.

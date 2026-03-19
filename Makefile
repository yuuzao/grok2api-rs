IMAGE ?= yuuzao/grok2api-rs
VCS_REF ?= $(shell git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)
TAG_ARGS := -t $(IMAGE):$(VCS_REF) -t $(IMAGE):latest

.PHONY: build push

build:
	docker build \
		--build-arg VCS_REF=$(VCS_REF) \
		$(TAG_ARGS) \
		.

push:
	docker push $(IMAGE):$(VCS_REF)
	docker push $(IMAGE):latest

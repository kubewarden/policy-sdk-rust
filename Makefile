KUBE_API_VERSION?=1.26

.PHONY: fmt
fmt:
	K8S_OPENAPI_ENABLED_VERSION=$(KUBE_API_VERSION) cargo fmt --all -- --check

.PHONY: lint
lint:
	K8S_OPENAPI_ENABLED_VERSION=$(KUBE_API_VERSION) cargo clippy -- -D warnings

.PHONY: test
test: fmt lint
	cargo test
	cargo test --no-default-features

.PHONY: clean
clean:
	cargo clean

KUBE_API_VERSION?=1.31

.PHONY: fmt
fmt:
	K8S_OPENAPI_ENABLED_VERSION=$(KUBE_API_VERSION) cargo fmt --all -- --check

.PHONY: lint
lint:
	K8S_OPENAPI_ENABLED_VERSION=$(KUBE_API_VERSION) cargo clippy --all-features -- -D warnings

.PHONY: test
test: fmt lint
	@echo -e "\033[0;32mRun test with default features enabled\033[0m"
	cargo test

	@echo -e "\033[0;32mRun test with default features disabled\033[0m"
	cargo test --no-default-features

	@echo -e "\033[0;32mRun test with all features enabled\033[0m"
	cargo test --all-features

.PHONY: clean
clean:
	cargo clean

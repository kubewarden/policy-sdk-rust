KUBE_API_VERSION?=1.36
# Versions exercised by the *-all-versions targets (covers the CronJob spec boundary).
KUBE_API_VERSIONS?=1.35 1.36

# Exported so every recipe inherits the same Kubernetes API version selection.
export K8S_OPENAPI_ENABLED_VERSION=$(KUBE_API_VERSION)

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: doc
doc:
	RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc --all-features --no-deps

.PHONY: lint
lint:
	cargo clippy --all-features -- -D warnings

.PHONY: check
check:
	@echo -e "\033[0;32mCheck with default features enabled\033[0m"
	cargo check

	@echo -e "\033[0;32mCheck with default features disabled\033[0m"
	cargo check --no-default-features

	@echo -e "\033[0;32mCheck with all features enabled\033[0m"
	cargo check --all-features

.PHONY: check-all-versions
check-all-versions:
	@for v in $(KUBE_API_VERSIONS); do \
		echo -e "\033[0;32mcargo check against Kubernetes $$v\033[0m"; \
		$(MAKE) check KUBE_API_VERSION=$$v || exit 1; \
	done

.PHONY: test-suite
test-suite:
	@echo -e "\033[0;32mRun test with default features enabled\033[0m"
	cargo test

	@echo -e "\033[0;32mRun test with default features disabled\033[0m"
	cargo test --no-default-features

	@echo -e "\033[0;32mRun test with all features enabled\033[0m"
	cargo test --all-features

.PHONY: test-all-versions
test-all-versions:
	@for v in $(KUBE_API_VERSIONS); do \
		echo -e "\033[0;32mRun test suite against Kubernetes $$v\033[0m"; \
		$(MAKE) test-suite KUBE_API_VERSION=$$v || exit 1; \
	done

.PHONY: test
test: fmt lint test-suite

.PHONY: clean
clean:
	cargo clean

.PHONY: advisories
advisories:
	cargo deny check advisories

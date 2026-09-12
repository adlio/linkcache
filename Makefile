# linkcache build system
# Mirrors the checks run in CI (.github/workflows/ci.yml).
# Requires: cargo-llvm-cov (auto-installed by coverage targets if missing)

.DEFAULT_GOAL := help

.PHONY: help ensure-tools build release check fmt fmt-check clippy lint \
	test doc doc-check coverage coverage-html coverage-ci clean ci all \
	run cache build_workflow test_workflow

# Single-threaded tests: the Firefox tests set a process-wide environment
# variable, so they must not run concurrently.
TEST_FLAGS := --all-targets --all-features -- --test-threads 1

CARGO_LLVM_COV := $(shell command -v cargo-llvm-cov 2>/dev/null)

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

ensure-tools: ## Install required cargo tools if missing
ifndef CARGO_LLVM_COV
	@echo "Installing cargo-llvm-cov..."
	@cargo install cargo-llvm-cov --locked
endif

build: ## Build all targets (debug)
	cargo build --all-targets --all-features

release: ## Build all targets (release)
	cargo build --all-targets --all-features --release

check: ## Type-check all targets
	cargo check --all-targets --all-features

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting (CI)
	cargo fmt --all -- --check

clippy: ## Lint with clippy, denying warnings (CI)
	cargo clippy --all-targets --all-features -- -D warnings

lint: fmt clippy ## Format and lint

test: ## Run all tests
	cargo test $(TEST_FLAGS)

doc: ## Generate and open docs
	cargo doc --no-deps --all-features --open

doc-check: ## Build docs, denying warnings (CI)
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

coverage: ensure-tools ## Show coverage summary in the console
	cargo llvm-cov --all-features --tests --show-missing-lines -- --test-threads 1

coverage-html: ensure-tools ## Generate an HTML coverage report and open it
	cargo llvm-cov --all-features --tests --html --open -- --test-threads 1

coverage-ci: ensure-tools ## Generate LCOV coverage for CI
	cargo llvm-cov clean --workspace
	cargo llvm-cov --all-features --tests --lcov --output-path lcov.info -- --test-threads 1

clean: ## Remove build and coverage artifacts
	cargo clean

ci: fmt-check clippy build doc-check test ## Run the full CI pipeline locally

all: lint build test ## Format, lint, build, and test

# ---------------------------------------------------------------------------
# Project-specific targets (the linkcache Alfred workflow binary)
# ---------------------------------------------------------------------------

run: ## Build and run the binary
	cargo build --features="bin" && ./target/debug/linkcache

cache: ## Build and run the binary with --cache
	cargo build --features="bin" && ./target/debug/linkcache --cache

build_workflow: release ## Build the binary and stage it under target/workflow
	mkdir -p target/workflow && cp ./target/release/linkcache ./target/workflow

test_workflow: build_workflow ## Run the staged workflow binary against test data
	alfred_workflow_data=./test_workflow/workflow_data \
	alfred_workflow_cache=./test_workflow/workflow_cache \
	./target/workflow/linkcache test

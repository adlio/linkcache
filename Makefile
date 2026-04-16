# linkcache Build System
# Requires: cargo-nextest, cargo-llvm-cov (auto-installed if missing)

.DEFAULT_GOAL := help

.PHONY: help test coverage coverage-html build build-release install clean fmt fmt-check lint check doc doc-check all ci ensure-tools run cache build_workflow test_workflow

# Tool installation helpers
CARGO_NEXTEST := $(shell command -v cargo-nextest 2>/dev/null)
CARGO_LLVM_COV := $(shell command -v cargo-llvm-cov 2>/dev/null)

ensure-tools: ## Install required cargo tools if missing
ifndef CARGO_NEXTEST
	@echo "Installing cargo-nextest..."
	@cargo install cargo-nextest --locked
endif
ifndef CARGO_LLVM_COV
	@echo "Installing cargo-llvm-cov..."
	@cargo install cargo-llvm-cov --locked
endif

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

test: ensure-tools ## Run tests with nextest
	cargo nextest run --all-features

coverage: ensure-tools ## Show coverage summary in console
	cargo llvm-cov nextest --all-features

coverage-html: ensure-tools ## Generate HTML coverage report and open
	cargo llvm-cov nextest --all-features --html --open

build: ## Build debug
	cargo build --all-targets --all-features

build-release: ## Build release
	cargo build --all-targets --all-features --release

install: ## Install linkcache to ~/.cargo/bin
	cargo install --path .

check: ## Run cargo check
	cargo check --all-targets --all-features

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all -- --check

lint: ## Run clippy
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean

doc: ## Generate docs
	cargo doc --no-deps --open

doc-check: ## Check docs build without warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

run: ## Build and run linkcache binary
	cargo build --features="bin" && \
	./target/debug/linkcache

cache: ## Build and run linkcache with --cache flag
	cargo build --features="bin" && \
	./target/debug/linkcache --cache

build_workflow: build-release ## Build workflow binary into target/workflow
	mkdir -p target/workflow && \
	cp ./target/release/linkcache ./target/workflow

test_workflow: build_workflow ## Run workflow test with test environment
	alfred_workflow_data=./test_workflow/workflow_data \
	alfred_workflow_cache=./test_workflow/workflow_cache \
	./target/workflow/linkcache test

all: ensure-tools fmt lint build test ## Format, lint, build, and test

ci: ensure-tools fmt-check lint build doc-check test ## Check formatting, lint, build, docs, test (for CI/hooks)

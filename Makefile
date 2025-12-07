.PHONY: build test clean install run fmt lint check help

# Default target
.DEFAULT_GOAL := help

# Project configuration
PROJECT_NAME := rusty-dns
CARGO := cargo
INSTALL_PATH := /usr/local/bin

## Build the project in release mode
build:
	@echo "Building $(PROJECT_NAME)..."
	$(CARGO) build --release

## Build in debug mode
build-debug:
	@echo "Building $(PROJECT_NAME) in debug mode..."
	$(CARGO) build

## Run all tests
test:
	@echo "Running tests..."
	$(CARGO) test

## Run tests with output
test-verbose:
	@echo "Running tests (verbose)..."
	$(CARGO) test -- --nocapture

## Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf target/

## Install dependencies (Rust doesn't need this, but kept for consistency)
install:
	@echo "Checking Rust toolchain..."
	@rustc --version
	@cargo --version

## Install the binary to system
install-bin: build
	@echo "Installing $(PROJECT_NAME) to $(INSTALL_PATH)..."
	@cp target/release/rusty-dns $(INSTALL_PATH)/
	@echo "Installed successfully!"

## Run the server (help command)
run:
	@echo "Running $(PROJECT_NAME)..."
	$(CARGO) run -- --help

## Format code
fmt:
	@echo "Formatting code..."
	$(CARGO) fmt --all

## Check formatting without making changes
fmt-check:
	@echo "Checking code formatting..."
	$(CARGO) fmt --all -- --check

## Run clippy for linting
lint:
	@echo "Running clippy..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

## Run all checks (fmt, lint, test)
check: fmt-check lint test
	@echo "All checks passed!"

## Run cargo check (fast compile check)
check-build:
	@echo "Checking build..."
	$(CARGO) check --all-targets

## Build documentation
doc:
	@echo "Building documentation..."
	$(CARGO) doc --no-deps --open

## Show this help message
help:
	@echo "$(PROJECT_NAME) - Dynamic DNS with MCP support"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Available targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## /  /'

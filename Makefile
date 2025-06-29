# Fathom to Loom - Development Commands

.PHONY: help dev test build build-docker clean lint fix setup status

# Default target
help:
	@echo "Available commands:"
	@echo "  help         - Show this help message"
	@echo "  dev          - Start development environment"
	@echo "  test         - Run all tests"
	@echo "  build        - Build all crates in release mode"
	@echo "  build-docker - Build Docker images"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run clippy and check formatting"
	@echo "  fix          - Fix clippy warnings and format code"
	@echo "  setup        - Setup development environment"
	@echo "  status       - Show project status"

# Development environment
dev:
	@echo "Starting development environment..."
	@echo "Note: Run each service manually:"
	@echo "  Terminal 1: cargo run --bin backend"
	@echo "  Terminal 2: cargo run --bin worker"  
	@echo "  Terminal 3: cargo run --bin fathom-loom-frontend"
	@echo "  Terminal 4: docker-compose up -d"

# Test all crates
test:
	cargo test --workspace

# Test with coverage (requires tarpaulin)
test-coverage:
	cargo tarpaulin --workspace --out html --output-dir coverage

# Lint and format
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo fmt --all --check

# Fix linting and formatting issues
fix:
	cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	cargo fmt --all

# Build all crates in release mode
build:
	cargo build --workspace --release

# Build Docker images (requires Dockerfiles)
build-docker:
	@echo "Building Docker images..."
	@echo "Note: Create Dockerfiles first:"
	@echo "  backend.Dockerfile"
	@echo "  frontend.Dockerfile" 
	@echo "  worker.Dockerfile"
	# docker build -t fathom-to-loom-backend -f backend.Dockerfile .
	# docker build -t fathom-to-loom-frontend -f frontend.Dockerfile .
	# docker build -t fathom-to-loom-worker -f worker.Dockerfile .

# Clean build artifacts
clean:
	cargo clean
	# docker system prune -f

# Setup development environment
setup:
	@echo "Setting up development environment..."
	rustup component add clippy rustfmt
	# cargo install cargo-watch dx trunk wasm-pack
	@if exist .env.example copy .env.example .env
	@echo "Please edit .env with your configuration"

# Show project status
status:
	@echo "=== Cargo Check ==="
	cargo check --workspace
	@echo ""
	@echo "=== Git Status ==="
	git status --short

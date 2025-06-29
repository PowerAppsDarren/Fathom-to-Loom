# Fathom to Loom - Development Commands

# Default recipe
default:
    @just --list

# Development commands
dev:
    @echo "Starting development environment..."
    @just backend-dev &
    @just frontend-dev &
    @just worker-dev &
    docker-compose up -d pocketbase smtp-service

# Start backend development server
backend-dev:
    cd backend && cargo watch -x "run --bin backend"

# Start frontend development server  
frontend-dev:
    cd frontend && dx serve --platform web --hot-reload

# Start worker in development mode
worker-dev:
    cd worker && cargo watch -x "run --bin worker"

# Test all crates
test:
    cargo test --workspace

# Test with coverage
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

# Build Docker images
build-docker:
    @echo "Building Docker images..."
    docker build -t fathom-to-loom-backend -f backend.Dockerfile . 
    docker build -t fathom-to-loom-frontend -f frontend.Dockerfile .
    docker build -t fathom-to-loom-worker -f worker.Dockerfile .

# Clean build artifacts
clean:
    cargo clean
    docker system prune -f

# Run database migrations (when implemented)
migrate:
    @echo "Running database migrations..."
    # TODO: Add migration command when implemented

# Setup development environment
setup:
    @echo "Setting up development environment..."
    rustup component add clippy rustfmt
    cargo install cargo-watch dx trunk wasm-pack
    cp .env.example .env
    @echo "Please edit .env with your configuration"

# Deploy to production (placeholder)
deploy:
    @echo "Deploying to production..."
    @just build-docker
    # TODO: Add deployment logic

# Show project status
status:
    @echo "=== Cargo Check ==="
    cargo check --workspace
    @echo ""
    @echo "=== Docker Status ==="
    docker-compose ps
    @echo ""
    @echo "=== Git Status ==="
    git status --short

# Fathom-to-Loom

A Rust mono-repo application with microservices architecture, featuring automated per-user database management and secure task processing.

## Architecture Overview

### Services

- **Backend** (Axum + Tokio) - REST/WebSocket API server
- **Frontend** (Dioxus 0.6) - Web/WASM application
- **Worker** - Long-running task executor with shared state
- **Common** - Shared crate for models, DTOs, and encryption helpers

### Data Layer

- **Global PocketBase** (`pb_global`) - Manages user accounts and tracks user databases
- **Per-User PocketBase** - Individual database files created automatically on first login
- **User DB Naming Convention**: `yyyy-mm-dd-XXX-username`
  - `yyyy-mm-dd`: Creation date
  - `XXX`: Sequential number (001, 002, etc.)
  - `username`: Email prefix before @ sign

### Concurrency Control

- Single global queue collection with concurrency = 1
- Enforced by optimistic "claim & lock" mechanism in worker
- Ensures sequential task processing

### Security

- **AES-256-GCM** encryption with master key from `.env`
- **JWT-based** session management
- **PocketBase built-in** email/password authentication
- Future OAuth integration ready

### Project Structure
```
Fathom-to-Loom/
├── backend/           # Axum REST/WebSocket server
├── frontend/          # Dioxus web application
├── worker/            # Task processing service
├── common/            # Shared models and utilities
├── pb_migrations/     # PocketBase schema migrations
├── nginx/             # Reverse proxy configuration
├── logs/              # Application logs
├── docker-compose.yml # Container orchestration
├── .env.sample        # Environment template
└── .env               # Environment configuration
```

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust (for local development)

### Development Setup

1. **Clone and setup environment**:
   ```bash
   git clone <repo-url>
   cd Fathom-to-Loom
   cp .env.sample .env
   # Edit .env with your values, or use the generated .env for development
   ```

2. **Run with Docker Compose**:
   ```bash
   # Development mode
   docker-compose up -d
   
   # Production mode
   docker-compose --profile production up -d
   ```

3. **Access services**:
   - Frontend: http://localhost:8080
   - Backend API: http://localhost:3000
   - Global PocketBase Admin: http://localhost:8090/_/

### Local Development

1. **Install dependencies**:
   ```bash
   cargo install dioxus-cli
   cargo install cargo-watch
   ```

2. **Run services locally**:
   ```bash
   # Terminal 1: Start PocketBase
   docker-compose up pb_global
   
   # Terminal 2: Backend
   cd backend && cargo run
   
   # Terminal 3: Worker
   cd worker && cargo run
   
   # Terminal 4: Frontend
   cd frontend && dx serve --hot-reload
   ```

## Environment Configuration

### Required Variables

- `AES_MASTER_KEY`: 32-byte base64 encoded encryption key
- `JWT_SECRET`: JWT signing secret
- `PB_ENCRYPTION_KEY`: PocketBase encryption key
- `PB_ADMIN_PASSWORD`: PocketBase admin password

### Optional Variables

- `RUST_LOG`: Log level (debug, info, warn, error)
- `WORKER_CONCURRENCY`: Number of concurrent workers (default: 1)
- `QUEUE_POLL_INTERVAL`: Queue polling interval in seconds

## User Database Management

### Automatic User DB Creation

1. User registers/logs in via PocketBase authentication
2. Backend checks global PocketBase for existing user DB record
3. If none exists, creates new user DB file with naming convention
4. Registers user DB in global PocketBase tracking collection
5. Spins up user DB instance on dynamic port

### User DB Naming

Format: `yyyy-mm-dd-XXX-username`

Example: `2024-01-15-001-johndoe` for user `johndoe@example.com` created on Jan 15, 2024

## Task Processing

### Queue System

- Global queue collection in PocketBase
- Workers poll for available tasks
- Optimistic locking prevents duplicate processing
- Concurrency limited to 1 for sequential processing

### Worker Architecture

- Shares Axum application state via `Arc<AppState>`
- Communicates with backend via internal APIs
- Processes tasks with access to user-specific databases

## Authentication

### Current Implementation

- PocketBase built-in email/password authentication
- JWT tokens for session management
- CORS-enabled for frontend integration

### Future OAuth Integration

- Google OAuth (slots prepared)
- GitHub OAuth (slots prepared)
- Extensible for additional providers

## Deployment

### Docker Compose (Recommended)

```bash
# Development
docker-compose up -d

# Production with nginx
docker-compose --profile production up -d
```

### Manual Deployment

1. Build Rust services: `cargo build --release`
2. Build frontend: `dx build --release`
3. Configure nginx reverse proxy
4. Setup PocketBase with proper permissions
5. Configure environment variables

## Security Considerations

- All encryption keys are loaded from environment variables
- PocketBase admin credentials are configurable
- User data is isolated in separate database files
- AES-256-GCM encryption for sensitive data
- JWT tokens with configurable expiration

## Development Notes

- Mono-repo structure for easy dependency management
- Shared `common` crate for consistency
- Docker-first development workflow
- Comprehensive logging and monitoring ready
- Prepared for horizontal scaling

## Contributing

1. Follow existing code patterns
2. Update documentation for architectural changes
3. Test changes in Docker environment
4. Ensure security best practices

## License

[Your License Here]

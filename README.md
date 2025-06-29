# Fathom-to-Loom

A modern web application integrating Fathom analytics and Loom video capabilities with a robust admin system featuring SMTP configuration and email management.

## Architecture

### Technology Stack
- **Frontend**: Dioxus (Rust-based UI framework)
- **Backend**: PocketBase (Go-based BaaS with built-in admin UI)
- **SMTP Service**: Rust service with Axum and Lettre
- **Database**: PocketBase's embedded SQLite
- **Configuration**: Environment-based with PocketBase collections fallback

### Project Structure
```
Fathom-to-Loom/
├── frontend/           # Dioxus web application
├── smtp-service/       # Rust SMTP service
├── pocketbase/         # PocketBase configuration and schema
├── .env.example        # Environment configuration template
└── Cargo.toml          # Workspace configuration
```

## Features

### ✅ Completed
- [x] Project structure setup with Rust workspace
- [x] PocketBase schema for SMTP settings and email queue
- [x] Basic SMTP service architecture
- [x] Environment configuration management
- [x] SMTP configuration hierarchy (env → database)

### 🚧 In Progress
- [ ] SMTP service implementation (config, email, pocketbase, security modules)
- [ ] Dioxus frontend admin interface
- [ ] PocketBase integration and authentication

### 📋 Planned
- [ ] Email queue processing and retry mechanisms
- [ ] Admin authentication and authorization
- [ ] SMTP connection testing and validation
- [ ] Fathom analytics integration
- [ ] Loom video integration
- [ ] Real-time dashboard with analytics
- [ ] Comprehensive testing suite
- [ ] Docker deployment configuration

## Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **PocketBase** (latest version)
- **Node.js** (for Dioxus web builds)
- **Git**

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd Fathom-to-Loom
   ```

2. **Setup environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Install Rust dependencies**:
   ```bash
   cargo build
   ```

4. **Setup PocketBase**:
   ```bash
   # Download PocketBase binary for your platform
   # Import schema: pocketbase/pb_schema.json
   ```

### Development Setup

1. **Start PocketBase**:
   ```bash
   ./pocketbase serve --http=127.0.0.1:8090
   ```

2. **Start SMTP Service**:
   ```bash
   cargo run --bin fathom-loom-smtp-service
   ```

3. **Start Frontend** (when implemented):
   ```bash
   cd frontend
   dx serve --hot-reload
   ```

### Configuration

#### Environment Variables
- `POCKETBASE_URL`: PocketBase server URL
- `SMTP_SERVICE_HOST/PORT`: SMTP service binding
- `SMTP_*`: Fallback SMTP configuration
- `ENCRYPTION_KEY`: For sensitive data encryption

#### Admin Configuration
1. Access PocketBase admin at `http://localhost:8090/_/`
2. Create admin account
3. Configure SMTP settings through admin interface
4. Test SMTP connection

## API Endpoints

### SMTP Service
- `GET /health` - Service health check
- `POST /send-email` - Queue email for sending
- `POST /test-smtp` - Test SMTP connection

### PocketBase Collections
- `smtp_settings` - SMTP configuration
- `email_queue` - Email queue management

## Security Best Practices

- **Encryption**: Sensitive SMTP credentials encrypted at rest
- **Authentication**: Role-based access control via PocketBase
- **Validation**: Input validation and sanitization
- **Rate Limiting**: SMTP operations rate limited
- **Audit Logging**: Configuration changes logged
- **Environment Isolation**: Separate configs for dev/prod

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow Rust best practices and run `cargo fmt` and `cargo clippy`
4. Add tests for new functionality
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

- Project Link: [https://github.com/username/Fathom-to-Loom](https://github.com/username/Fathom-to-Loom)

## Acknowledgments

- **Dioxus**: Modern Rust UI framework
- **PocketBase**: Excellent BaaS solution
- **Lettre**: Robust Rust SMTP library
- **Axum**: Fast and ergonomic web framework

# Environment Configuration Guide

This document describes how to configure the Fathom to Loom application using environment variables.

## Quick Start

1. Copy the sample environment file:
   ```bash
   cp .env.sample .env
   ```

2. Update the secret values in `.env` (see [Security Configuration](#security-configuration) below)

3. Start the application:
   ```bash
   docker-compose up
   ```

## Environment Variables

### Core Security Variables

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MASTER_KEY` | AES-256 encryption key (32 bytes, base64) | `FmbwJVUUZp/7tDAl00IfNO/FQimAax+zjYZWIx3I5ho=` | ✅ |
| `JWT_SECRET` | JWT token signing secret | `CHANGE_ME_JWT_SECRET_32B` | ✅ |
| `PB_ENCRYPTION_KEY` | PocketBase database encryption key | `IF614Fvr/psR3FqywPWbZrMeAGOTCiHZyxQt1d0lFHU=` | ✅ |

### PocketBase Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `GLOBAL_PB_URL` | PocketBase server URL | `http://pb_global:8090` | ✅ |
| `GLOBAL_PB_ADMIN_EMAIL` | Admin email for PocketBase | `admin@example.com` | ✅ |
| `GLOBAL_PB_ADMIN_PW` | Admin password for PocketBase | `L2=dAcl!R@>E6H$>}_]&qmpy` | ✅ |

### Application Settings

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `RUST_LOG` | Logging level | `info` | `error`, `warn`, `info`, `debug`, `trace` |
| `QUEUE_CONCURRENCY` | Worker queue concurrency | `1` | Any positive integer |
| `WORKER_CONCURRENCY` | Max simultaneous worker tasks | `1` | Any positive integer |
| `QUEUE_POLL_INTERVAL` | Worker polling interval (seconds) | `5` | Any positive integer |

### Network & Ports

| Variable | Description | Default |
|----------|-------------|---------|
| `BACKEND_PORT` | Backend API server port | `3000` |
| `FRONTEND_PORT` | Frontend development server port | `8080` |
| `PB_GLOBAL_PORT` | PocketBase external port | `8090` |
| `HTTP_PORT` | Production HTTP port | `80` |
| `HTTPS_PORT` | Production HTTPS port | `443` |

### CORS & API

| Variable | Description | Default |
|----------|-------------|---------|
| `CORS_ORIGINS` | Allowed CORS origins (comma-separated) | `http://localhost:8080,http://localhost:3000` |
| `API_BASE_URL` | API base URL for frontend | `http://localhost:3000` |

## Security Configuration

### Generating Secure Keys

**Master Key & PocketBase Encryption Key:**
```bash
# Using OpenSSL
openssl rand -base64 32

# Using PowerShell (Windows)
[Convert]::ToBase64String((1..32 | ForEach { Get-Random -Maximum 256 }))
```

**JWT Secret:**
```bash
# Generate a strong JWT secret
openssl rand -base64 64
```

**Admin Password:**
```bash
# Generate a strong password with special characters
pwgen -s 24 1
```

### Security Best Practices

1. **Never commit `.env` files** - they contain sensitive secrets
2. **Use different secrets for each environment** (dev, staging, prod)
3. **Rotate secrets regularly** in production
4. **Use environment-specific key management** in production (e.g., AWS Secrets Manager, Azure Key Vault)

## Environment-Specific Configuration

### Development

Copy `.env.sample` to `.env` and update the required secrets. The default values work for local development.

### Production

1. **Set strong, unique secrets** for all security variables
2. **Configure proper hostnames** instead of localhost
3. **Use HTTPS** for all external connections
4. **Enable proper logging levels** (usually `info` or `warn`)
5. **Set appropriate concurrency** based on server resources

### Docker Configuration

The application uses Docker Compose with the following environment variable precedence:

1. **Environment variables set in shell**
2. **Variables from `.env` file**
3. **Default values in `docker-compose.yml`**

## Configuration Loading

### Backend & Worker (Rust)

Both backend and worker use the `dotenvy` crate to load environment variables:

```rust
// Load from .env file
dotenvy::dotenv().ok();

// Load configuration
let config = Config::from_env()?;
```

### Frontend (Dioxus/WASM)

The frontend can get configuration in two ways:

1. **Build-time variables** - compiled into the WASM binary
2. **Runtime API call** - fetched from `/api/env` endpoint

Build-time variables (set during `cargo build`):
```bash
API_BASE_URL=http://localhost:3000 cargo build --release
```

## API Endpoints

### `/api/env`

Returns safe, non-sensitive configuration values for the frontend:

```json
{
  "api": {
    "base_url": "http://localhost:3000",
    "version": "0.1.0"
  },
  "database": {
    "url": "http://pb_global:8090"
  },
  "logging": {
    "level": "info"
  },
  "cors": {
    "origins": ["http://localhost:8080", "http://localhost:3000"]
  },
  "features": {
    "auth_enabled": true,
    "encryption_enabled": true
  }
}
```

## Troubleshooting

### Common Issues

1. **"Environment variable required" errors**
   - Ensure all required variables are set in `.env`
   - Check for typos in variable names

2. **Database connection errors**
   - Verify `GLOBAL_PB_URL` is correct
   - Ensure PocketBase container is running
   - Check network connectivity between containers

3. **Authentication errors**
   - Verify `GLOBAL_PB_ADMIN_EMAIL` and `GLOBAL_PB_ADMIN_PW` are correct
   - Ensure admin user exists in PocketBase

4. **Frontend can't reach backend**
   - Check `API_BASE_URL` configuration
   - Verify CORS origins include the frontend URL
   - Ensure backend is running and accessible

### Validation

Run the environment validation script:

```bash
chmod +x test_env.sh
./test_env.sh
```

This will check:
- ✅ All required variables are present
- ✅ Secrets appear to be properly generated
- ✅ All components compile successfully

## Migration from Previous Versions

If upgrading from a previous version, note these variable name changes:

| Old Name | New Name | Notes |
|----------|----------|-------|
| `AES_MASTER_KEY` | `MASTER_KEY` | Both names supported for compatibility |
| `PB_ADMIN_EMAIL` | `GLOBAL_PB_ADMIN_EMAIL` | Both names supported |
| `PB_ADMIN_PASSWORD` | `GLOBAL_PB_ADMIN_PW` | Both names supported |
| `DATABASE_URL` | `GLOBAL_PB_URL` | Both names supported |

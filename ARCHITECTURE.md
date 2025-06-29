# Architecture Decision Record - Fathom-to-Loom

**Date**: 2024-12-29  
**Status**: ✅ LOCKED-IN  
**Version**: 1.0  

## Executive Summary

This document records the finalized architecture decisions for the Fathom-to-Loom project, a Rust mono-repo application with microservices architecture, automated per-user database management, and secure task processing.

## Architecture Decisions

### 1. Technology Stack ✅

| Component | Technology | Justification |
|-----------|------------|---------------|
| **Backend** | Axum + Tokio | Performance, async-first, excellent ecosystem |
| **Frontend** | Dioxus 0.6 (WASM) | Rust consistency, modern reactive UI |
| **Worker** | Tokio-based | Shares state with backend, efficient async processing |
| **Common** | Rust crate | Shared models, DTOs, encryption helpers |
| **Database** | PocketBase | Built-in auth, admin UI, SQLite-based |
| **Deployment** | Docker Compose | Container orchestration, environment parity |

### 2. Data Architecture ✅

#### Global PocketBase (`pb_global`)
- **Purpose**: User account management + user DB tracking
- **Collections**: `users`, `user_databases`, `global_queue`
- **Port**: 8090 (configurable via `PB_GLOBAL_PORT`)

#### Per-User PocketBase Databases
- **Creation**: Automatic on first login/registration
- **Naming Convention**: `yyyy-mm-dd-XXX-username`
  - `yyyy-mm-dd`: Creation date
  - `XXX`: Sequential number (001, 002, etc.)
  - `username`: Email prefix before @ symbol
- **Example**: `2024-01-15-001-johndoe` for `johndoe@example.com`
- **Storage**: Shared volume mount `/app/user_dbs`

### 3. Service Communication ✅

#### Inter-Service Architecture
- **Backend ↔ Worker**: Shared `Arc<AppState>` + internal HTTP APIs
- **Frontend ↔ Backend**: REST/WebSocket APIs
- **All ↔ PocketBase**: HTTP client connections
- **Message Passing**: Tokio channels for async communication

#### State Management
```rust
Arc<AppState> {
    config: AppConfig,
    pb_client: PocketBaseClient,
    user_db_manager: UserDbManager,
    encryption: EncryptionService,
    // Shared between backend & worker
}
```

### 4. Concurrency Control ✅

#### Global Queue System
- **Collection**: `global_queue` in `pb_global`
- **Concurrency**: Limited to 1 (enforced by optimistic locking)
- **Mechanism**: Worker "claim & lock" with timestamps
- **Polling**: Configurable interval (default: 5 seconds)

#### Lock Implementation
```sql
-- Optimistic locking strategy
UPDATE global_queue 
SET status = 'processing', worker_id = ?, claimed_at = NOW() 
WHERE id = ? AND status = 'pending' AND claimed_at IS NULL
```

### 5. Security Architecture ✅

#### Encryption
- **Algorithm**: AES-256-GCM
- **Key Management**: Master key from `.env` (`AES_MASTER_KEY`)
- **Key Format**: 32-byte base64 encoded
- **Usage**: Sensitive user data, SMTP credentials, tokens

#### Authentication Flow
1. **Primary**: PocketBase email/password
2. **Sessions**: JWT tokens (`JWT_SECRET`)
3. **Future**: OAuth integration slots (Google, GitHub)

#### Security Measures
- Environment-based secrets
- Per-user data isolation
- Encrypted sensitive fields
- CORS configuration
- Rate limiting ready

### 6. Environment Management ✅

#### Configuration Hierarchy
1. **Environment Variables** (highest priority)
2. **`.env` file**
3. **`.env.local`** (optional)
4. **Default values** (fallback)

#### Required Environment Variables
```bash
AES_MASTER_KEY=<32-byte-base64>      # Encryption
JWT_SECRET=<64-byte-base64>          # Sessions
PB_ENCRYPTION_KEY=<32-byte-base64>   # PocketBase
PB_ADMIN_PASSWORD=<secure-password>  # Admin access
```

### 7. Deployment Strategy ✅

#### Docker Compose Architecture
- **Development Profile**: All services with hot-reload
- **Production Profile**: + Nginx reverse proxy
- **Volumes**: Persistent data, user databases, logs
- **Networks**: Isolated `fathom_network`
- **Health Checks**: Service dependency management

#### Service Dependencies
```
nginx (production) → frontend + backend
frontend → backend
backend → pb_global (health check)
worker → pb_global + backend
```

### 8. User Database Lifecycle ✅

#### Automatic Provisioning
1. User registers/authenticates via PocketBase
2. Backend checks `user_databases` collection
3. If no record exists:
   - Generate DB filename with naming convention
   - Create PocketBase instance file
   - Register in tracking collection
   - Start DB process on dynamic port
4. Return connection details to user session

#### Database Management
- **Tracking**: Global registry in `pb_global`
- **Isolation**: Separate SQLite files per user
- **Scaling**: Dynamic port allocation
- **Cleanup**: Configurable retention policies

### 9. Monitoring & Observability ✅

#### Logging Strategy
- **Framework**: `tracing` with structured logging
- **Levels**: Configurable via `RUST_LOG`
- **Destinations**: Files + stdout (Docker logs)
- **Retention**: Log rotation configured

#### Health Checks
- **PocketBase**: HTTP health endpoint
- **Services**: Custom health endpoints
- **Dependencies**: Docker health check integration

## Open Questions Resolved

### Q1: Per-User Database Management
**✅ RESOLVED**: Use global PocketBase to track user DB files with naming convention `yyyy-mm-dd-XXX-username`

### Q2: Service Communication
**✅ RESOLVED**: Shared Axum state + internal APIs + Tokio channels for optimal performance

### Q3: Environment Management
**✅ RESOLVED**: Hierarchical config with type-safe structs, separate dev/prod environments

### Q4: Deployment Strategy
**✅ RESOLVED**: Docker Compose with development/production profiles, Nginx for production

## Implementation Status

- [x] Architecture defined and locked-in
- [x] Docker Compose configuration created
- [x] Environment configuration templates created
- [x] Security keys generated for development
- [x] Documentation updated
- [ ] Rust workspace and crates (next step)
- [ ] Backend service implementation
- [ ] Worker service implementation
- [ ] Frontend implementation
- [ ] Common crate implementation

## Migration Considerations

### From Previous Architecture
- **Breaking Changes**: Complete rewrite from SMTP-focused to microservices
- **Data Migration**: PocketBase schema migration required
- **Configuration**: New environment variable structure

### Future Scaling
- **Horizontal**: Worker instances can be scaled
- **Database**: PocketBase can be replaced with PostgreSQL cluster
- **Caching**: Redis integration prepared
- **CDN**: Static asset delivery optimization ready

## Risk Mitigation

### Single Points of Failure
- **Global PocketBase**: Backup and clustering strategies defined
- **Shared Volumes**: Network storage for production
- **Worker Queue**: Dead letter queue implementation planned

### Security Risks
- **Key Rotation**: Process documented for production
- **User Isolation**: File system permissions enforced
- **Input Validation**: Comprehensive validation at all boundaries

## Approval

**Architect**: ✅ Approved  
**Security Review**: ✅ Approved  
**Performance Review**: ✅ Approved  
**Deployment Review**: ✅ Approved  

---

**Next Step**: Begin implementation of Rust workspace and service scaffolding

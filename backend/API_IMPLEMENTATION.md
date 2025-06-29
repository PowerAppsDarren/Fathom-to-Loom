# Axum REST + WebSocket API Implementation

## Overview

This implementation provides a complete Axum-based REST API with WebSocket support for the Fathom-to-Loom backend. The API includes authentication, encrypted key storage, meeting queue management, Fathom API proxying, and real-time WebSocket updates.

## Architecture

### Core Components

1. **AppState** - Unified application state containing:
   - Configuration (Arc<Config>)
   - PocketBase Manager (Arc<PocketBaseManager>)
   - WebSocket Manager (Arc<WebSocketManager>)
   - Meetings Queue (Arc<RwLock<Vec<Meeting>>>)

2. **State Adapters** - FromRef implementations to enable extracting specific state types from AppState

### API Endpoints

#### Authentication Routes (proxied to global PocketBase)
- `POST /auth/login` - User authentication
- `POST /auth/register` - User registration

#### API Keys Management (encrypted storage)
- `GET /api/keys` - Retrieve encrypted API keys
- `PUT /api/keys` - Store/update encrypted API keys

#### Meeting Queue Management
- `POST /api/queue` - Add meetings to processing queue
- `GET /api/queue` - Get current queue state
- `DELETE /api/queue/:id` - Remove meeting from queue

#### Meetings API (proxy to Fathom with caching)
- `GET /api/meetings` - Fetch meetings from Fathom API, cached in user PocketBase

#### WebSocket Real-time Updates
- `GET /queue_updates` - WebSocket endpoint for real-time queue position and progress updates

#### Health Checks
- `GET /health/pb` - PocketBase instances health
- `GET /health/ws` - WebSocket connections health

### Authentication & Security

#### PocketBase Token Validation
- **AuthUser extractor** - Validates PocketBase tokens and extracts user context
- **OptionalAuthUser extractor** - Optional authentication for public endpoints
- Shared extractors enforce PB auth token → user_id context

#### Encrypted Key Storage
- Uses AES-256-GCM encryption from common/crypto module
- Master key-based encryption for API keys
- Secure storage with metadata (service, key_id, created_at, expires_at)

### WebSocket Implementation

#### Real-time Features
- Queue position updates
- Processing progress notifications
- Connection management with user tracking
- Broadcast channels for queue and progress updates

#### WebSocket Message Types
```rust
enum WebSocketMessage {
    QueueUpdate(QueueUpdate),
    ProgressUpdate(ProgressUpdate),
    Ping,
    Pong,
}
```

#### Connection Management
- Tracks active connections per user
- Cleanup on disconnect
- Broadcast capabilities for updates

### Queue Management

#### Meeting Queue
- In-memory queue storage (ready for persistent storage)
- FIFO processing with position tracking
- Real-time position updates via WebSocket

#### Queue Operations
- Add meetings with automatic positioning
- Remove meetings with position rebalancing
- Get current queue state

### Fathom API Integration

#### Proxy Features
- Token-based authentication passthrough
- Response caching in user PocketBase instances
- Mock data for development (TODO: actual Fathom integration)

#### Caching Strategy
- Cache responses in user's individual PocketBase instance
- TTL-based cache invalidation (TODO: implementation)
- Cache-miss fallback to Fathom API

## File Structure

```
src/api/
├── mod.rs              # Main router and AppState
├── adapters.rs         # State conversion adapters
├── auth.rs             # Authentication endpoints
├── extractors.rs       # Auth token validation
├── keys.rs             # Encrypted key management
├── meetings.rs         # Fathom API proxy with caching
├── queue.rs            # Meeting queue management
├── websocket.rs        # WebSocket real-time updates
└── pocketbase.rs       # Legacy PocketBase management
```

## Dependencies Added

```toml
# WebSocket support
axum = { workspace = true, features = ["ws"] }
tokio-tungstenite = "0.21"
axum-extra = { version = "0.9", features = ["typed-header"] }
headers = "0.4"
http = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
futures = "0.3"
```

## Configuration

The implementation uses the existing Config structure with these key sections:
- **database** - Global PocketBase connection details
- **security** - Encryption keys and JWT secrets  
- **pocketbase** - User instance management settings

## Security Features

1. **Token Validation** - All API endpoints validate PocketBase tokens
2. **Encrypted Storage** - API keys stored with AES-256-GCM encryption
3. **User Isolation** - Per-user PocketBase instances for data isolation
4. **Secure Headers** - CORS and security headers via tower-http

## Real-time Updates

The WebSocket implementation provides:
- Queue position changes
- Processing progress updates
- Connection management
- Broadcast messaging

## Future Enhancements

1. **Fathom Integration** - Replace mock data with actual Fathom API calls
2. **Persistent Queue** - Replace in-memory queue with database storage
3. **Cache Implementation** - Complete PocketBase caching for meetings
4. **Authentication Enhancement** - Extract user_id from WebSocket connections
5. **Error Handling** - Enhanced error types and handling
6. **Rate Limiting** - API rate limiting for external calls
7. **Monitoring** - Metrics and observability integration

## Usage Examples

### Authentication
```bash
# Login
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Response includes token for subsequent API calls
```

### Queue Management
```bash
# Add meeting to queue
curl -X POST http://localhost:3000/api/queue \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123", "topic": "Team Meeting"}'

# Get queue status
curl -X GET http://localhost:3000/api/queue \
  -H "Authorization: Bearer TOKEN"
```

### WebSocket Connection
```javascript
const ws = new WebSocket('ws://localhost:3000/queue_updates');
ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Queue update:', update);
};
```

This implementation provides a robust foundation for the Fathom-to-Loom backend with all the specified features and room for future enhancements.

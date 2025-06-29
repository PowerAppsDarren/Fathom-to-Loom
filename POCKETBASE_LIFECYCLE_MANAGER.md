# PocketBase Lifecycle Manager

## Overview

The PocketBase Lifecycle Manager is a comprehensive system for managing per-user PocketBase instances in the Fathom-to-Loom application. It provides automatic lifecycle management, health monitoring, and REST API endpoints for controlling individual PocketBase instances.

## Features

### Core Functionality
- **Per-user PocketBase instances**: Each user gets their own isolated PocketBase database
- **Automatic port allocation**: Smart port assignment with collision detection (base_port + user_id_hash % 1000)
- **Process management**: Uses `tokio::process` for subprocess lifecycle management
- **Health monitoring**: Continuous monitoring with auto-restart capabilities
- **Database isolation**: Individual SQLite databases stored as `./user_dbs/pb_user_{id}.db`

### API Endpoints

#### `POST /api/users/{id}/init_pb`
Initialize a PocketBase instance for a specific user.

**Request Body:**
```json
{
  "force_restart": false
}
```

**Response:**
```json
{
  "success": true,
  "message": "PocketBase instance initialized on port 9123",
  "instance": {
    "user_id": "user_123",
    "port": 9123,
    "db_path": "./user_dbs/pb_user_user_123.db",
    "url": "http://localhost:9123",
    "status": "Running",
    "created_at": "2024-01-15T10:30:00Z",
    "last_health_check": "2024-01-15T10:30:30Z"
  }
}
```

#### `GET /api/users/{id}/pb_status`
Get the current status of a user's PocketBase instance.

**Response:**
```json
{
  "user_id": "user_123",
  "instance": {
    "user_id": "user_123",
    "port": 9123,
    "db_path": "./user_dbs/pb_user_user_123.db",
    "url": "http://localhost:9123",
    "status": "Running",
    "created_at": "2024-01-15T10:30:00Z",
    "last_health_check": "2024-01-15T10:30:30Z"
  }
}
```

#### `POST /api/users/{id}/stop_pb`
Stop a user's PocketBase instance.

**Response:**
```json
{
  "success": true,
  "message": "PocketBase instance stopped successfully"
}
```

#### `GET /api/pb_instances`
List all PocketBase instances across all users.

**Response:**
```json
{
  "instances": {
    "user_123": {
      "user_id": "user_123",
      "port": 9123,
      "url": "http://localhost:9123",
      "status": "Running",
      "created_at": "2024-01-15T10:30:00Z",
      "last_health_check": "2024-01-15T10:30:30Z"
    }
  },
  "count": 1
}
```

#### `GET /health/pb`
Health check endpoint for the PocketBase manager.

**Response:**
```json
{
  "status": "ok",
  "total_instances": 3,
  "running_instances": 2,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Architecture

### Components

1. **PocketBaseManager**: Core manager class handling lifecycle operations
2. **PocketBaseInstance**: Data structure representing a running instance
3. **API Layer**: REST endpoints for external interaction
4. **Health Monitor**: Background task for continuous health checking

### Instance States
- `Starting`: Instance is being initialized
- `Running`: Instance is healthy and operational
- `Failed`: Instance has crashed or failed health checks
- `Stopped`: Instance has been intentionally stopped

### Port Allocation Strategy
1. Calculate preferred port: `base_port + (hash(user_id) % 1000)`
2. If preferred port is unavailable, scan for next available port
3. Track allocated ports to prevent conflicts
4. Release ports when instances are stopped

### Health Monitoring
- **Process Monitoring**: Check if subprocess is still alive
- **HTTP Health Checks**: Ping PocketBase `/api/health` endpoint
- **Auto-restart**: Planned feature for failed instances
- **Monitoring Interval**: 30 seconds

## Configuration

### Environment Variables
```bash
# PocketBase binary path (defaults to "pocketbase")
PB_BINARY_PATH=/usr/local/bin/pocketbase

# Base port for allocation (defaults to 9000)
PB_BASE_PORT=9000

# Directory for user databases (defaults to "./user_dbs")
PB_USER_DBS_PATH=/app/user_dbs
```

### Directory Structure
```
user_dbs/
├── pb_user_123.db
├── pb_user_456.db
└── pb_user_789.db
```

## Usage Example

### Starting the Backend
```bash
# Set environment variables
export PB_BINARY_PATH=/usr/local/bin/pocketbase
export PB_BASE_PORT=9000
export PB_USER_DBS_PATH=./user_dbs

# Run the backend
cargo run --bin backend
```

### API Usage
```bash
# Initialize PocketBase for user
curl -X POST http://localhost:3000/api/users/test_user/init_pb \
  -H "Content-Type: application/json" \
  -d '{"force_restart": false}'

# Check status
curl http://localhost:3000/api/users/test_user/pb_status

# Stop instance
curl -X POST http://localhost:3000/api/users/test_user/stop_pb
```

### Integration with User Management
The PocketBase manager integrates with the existing user system by:
1. Using user IDs as keys for instance management
2. Storing connection info in user metadata (planned)
3. Providing per-user database isolation
4. Supporting authentication through user-specific instances

## Error Handling

### Common Errors
- **Port allocation failure**: When all ports in range are occupied
- **Process spawn failure**: When PocketBase binary is not found or fails to start
- **Health check failure**: When instance becomes unresponsive
- **IO errors**: When database directory creation fails

### Error Response Format
```json
{
  "success": false,
  "message": "Process error: Failed to spawn PocketBase process: No such file or directory (os error 2)",
  "instance": null
}
```

## Security Considerations

1. **Process Isolation**: Each instance runs as a separate process
2. **Database Isolation**: Individual SQLite files per user
3. **Port Security**: Instances bind to localhost only
4. **Process Management**: Automatic cleanup on termination
5. **Resource Limits**: Port range limitation (1000 ports max)

## Performance Characteristics

- **Memory Usage**: ~10-50MB per PocketBase instance
- **Startup Time**: ~2 seconds per instance
- **Port Range**: 1000 available ports (configurable)
- **Health Check Frequency**: 30 seconds
- **Concurrent Instances**: Limited by available ports and system resources

## Future Enhancements

1. **Auto-restart Logic**: Implement automatic restart for failed instances
2. **Resource Monitoring**: Track CPU and memory usage per instance
3. **Load Balancing**: Distribute instances across multiple servers
4. **Backup Management**: Automated backup and restore for user databases
5. **Metrics Collection**: Prometheus metrics for monitoring
6. **WebSocket Support**: Real-time status updates for instances

## Testing

Use the provided test script to validate functionality:
```bash
chmod +x test_pocketbase_api.sh
./test_pocketbase_api.sh
```

## Dependencies

- **tokio**: Async runtime and process management
- **axum**: Web framework for API endpoints
- **reqwest**: HTTP client for health checks
- **serde**: JSON serialization
- **chrono**: Timestamp management
- **tracing**: Logging and monitoring

## Troubleshooting

### Instance Won't Start
1. Check if PocketBase binary is in PATH or correctly configured
2. Verify port is not already in use
3. Check filesystem permissions for user_dbs directory
4. Review logs for specific error messages

### Health Check Failures
1. Verify PocketBase is responding on the allocated port
2. Check if instance process is still running
3. Test manual HTTP request to the health endpoint
4. Review network configuration and firewall settings

### Port Conflicts
1. Increase base port number to avoid conflicts
2. Check for other services using the port range
3. Monitor allocated_ports for proper cleanup
4. Consider expanding the port range if needed

This implementation provides a robust foundation for per-user PocketBase management with comprehensive monitoring, error handling, and API access.

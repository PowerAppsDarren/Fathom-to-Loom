# PocketBase Migrations

This directory contains PocketBase migration scripts that define the initial database schema for the Fathom-to-Loom application.

## Collections Created

### Built-in Collections
- `users` - PocketBase built-in authentication collection (automatically created)

### Custom Collections

#### 1. `user_meta`
Stores additional user metadata and status flags:
- `user_id` (relation to users) - References the authenticated user
- `file_path` (text) - Path to user's PocketBase database file
- `active` (boolean) - Whether the user account is active
- `verified` (boolean) - Whether the user's email is verified
- `premium` (boolean) - Whether the user has premium features

**Access Rules:**
- Users can only access their own records
- Authenticated users can create records
- Users can update/delete their own records

#### 2. `api_keys`
Stores encrypted API keys for external services:
- `user_id` (relation to users) - References the authenticated user
- `encrypted_fathom` (text) - Encrypted Fathom API key
- `encrypted_loom` (text) - Encrypted Loom API key

**Access Rules:**
- Users can only access their own API keys
- Authenticated users can create records
- Users can update/delete their own records

#### 3. `queue`
Manages job queue for processing meetings:
- `user_id` (relation to users) - References the authenticated user
- `meeting_id` (text) - Unique identifier for the meeting
- `status` (select) - Job status: pending, processing, completed, failed, cancelled
- `position` (number) - Position in queue
- `queued_at` (date) - When the job was queued
- `started_at` (date) - When processing started
- `completed_at` (date) - When processing completed
- `error_message` (text) - Error details if failed

**Access Rules:**
- Users can access their own queue items
- Admins can update/delete any queue items
- Regular users can only update their own items

#### 4. `logs`
Application logging system:
- `level` (select) - Log level: trace, debug, info, warn, error, fatal
- `msg` (text) - Log message
- `ts` (date) - Timestamp
- `user_id` (relation to users, optional) - Associated user if applicable
- `component` (text) - Component/service that generated the log
- `metadata` (json) - Additional structured data

**Access Rules:**
- Only admins can list/view all logs
- Users can view their own associated logs
- Both authenticated users and admins can create logs
- Only admins can update/delete logs

## Migration Process

The migration script `1730000000_initial_schema.js` will:

1. Create all required collections with proper schema definitions
2. Set up appropriate indexes for performance
3. Configure access rules for security
4. Provide rollback functionality

## Axum Admin Service Integration

The Axum admin service should:

1. **Check if collections exist** on startup using PocketBase API
2. **Apply migrations** if collections are missing
3. **Create initial admin user** if none exists
4. **Seed any required default data**

### Example Seeding Logic (Rust/Axum)

```rust
pub async fn seed_database() -> Result<(), Box<dyn std::error::Error>> {
    let pb_client = PocketBase::new("http://pb_global:8090");
    
    // Check if collections exist
    let collections = pb_client.collections().list().await?;
    
    let required_collections = ["user_meta", "api_keys", "queue", "logs"];
    let existing_collections: Vec<String> = collections
        .items
        .iter()
        .map(|c| c.name.clone())
        .collect();
    
    for collection in required_collections {
        if !existing_collections.contains(&collection.to_string()) {
            tracing::warn!("Collection {} not found. Please run migrations.", collection);
            // Optionally trigger migrations programmatically
        }
    }
    
    // Create admin user if needed
    if let Err(_) = pb_client.admins().list().await {
        create_initial_admin().await?;
    }
    
    Ok(())
}
```

## Environment Variables

Make sure these environment variables are set:

- `PB_ENCRYPTION_KEY` - Encryption key for PocketBase
- `PB_ADMIN_EMAIL` - Initial admin email
- `PB_ADMIN_PASSWORD` - Initial admin password

## Running Migrations

Migrations will be automatically applied when PocketBase starts up and detects new migration files in the `/pb_migrations` directory.

To manually apply migrations:
```bash
docker-compose exec pb_global ./pocketbase migrate
```

To rollback migrations:
```bash
docker-compose exec pb_global ./pocketbase migrate down
```

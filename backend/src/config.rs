use std::env;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
    pub pocketbase: PocketBaseConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub admin_email: String,
    pub admin_password: String,
    pub user_db_base_path: String,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub master_key: String,
    pub jwt_secret: String,
    pub pb_encryption_key: String,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PocketBaseConfig {
    pub base_port: u16,
    pub binary_path: String,
    pub user_dbs_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let server = ServerConfig {
            port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .or_else(|_| env::var("GLOBAL_PB_URL"))
                .unwrap_or_else(|_| "http://pb_global:8090".to_string()),
            admin_email: env::var("PB_ADMIN_EMAIL")
                .or_else(|_| env::var("GLOBAL_PB_ADMIN_EMAIL"))
                .unwrap_or_else(|_| "admin@example.com".to_string()),
            admin_password: env::var("PB_ADMIN_PASSWORD")
                .or_else(|_| env::var("GLOBAL_PB_ADMIN_PW"))
                .expect("PB_ADMIN_PASSWORD or GLOBAL_PB_ADMIN_PW must be set"),
            user_db_base_path: env::var("USER_DB_BASE_PATH")
                .unwrap_or_else(|_| "/app/user_dbs".to_string()),
        };

        let security = SecurityConfig {
            master_key: env::var("MASTER_KEY")
                .or_else(|_| env::var("AES_MASTER_KEY"))
                .expect("MASTER_KEY or AES_MASTER_KEY must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            pb_encryption_key: env::var("PB_ENCRYPTION_KEY")
                .expect("PB_ENCRYPTION_KEY must be set"),
        };

        let logging = LoggingConfig {
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        };

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:8080,http://localhost:3000".to_string());
        let cors = CorsConfig {
            origins: cors_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        };

        let pocketbase = PocketBaseConfig {
            base_port: env::var("PB_BASE_PORT")
                .unwrap_or_else(|_| "9000".to_string())
                .parse()?,
            binary_path: env::var("PB_BINARY_PATH")
                .unwrap_or_else(|_| "pocketbase".to_string()),
            user_dbs_path: env::var("PB_USER_DBS_PATH")
                .unwrap_or_else(|_| "./user_dbs".to_string()),
        };

        Ok(Config {
            server,
            database,
            security,
            logging,
            cors,
            pocketbase,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct EnvConfig {
    pub master_key: String,
    pub global_pb_url: String,
    pub global_pb_admin_email: String,
    pub global_pb_admin_pw: String,
    pub rust_log: String,
    pub queue_concurrency: u32,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self {
            master_key: env::var("MASTER_KEY")
                .expect("MASTER_KEY environment variable is required"),
            global_pb_url: env::var("GLOBAL_PB_URL")
                .unwrap_or_else(|_| "http://pb_global:8090".to_string()),
            global_pb_admin_email: env::var("GLOBAL_PB_ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@example.com".to_string()),
            global_pb_admin_pw: env::var("GLOBAL_PB_ADMIN_PW")
                .expect("GLOBAL_PB_ADMIN_PW environment variable is required"),
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            queue_concurrency: env::var("QUEUE_CONCURRENCY")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
        }
    }
}

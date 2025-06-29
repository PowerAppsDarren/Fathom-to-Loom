use std::env;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub worker: WorkerSettings,
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
    pub pb_encryption_key: String,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone)]
pub struct WorkerSettings {
    pub concurrency: u32,
    pub poll_interval: u64,
    pub queue_concurrency: u32,
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
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
            pb_encryption_key: env::var("PB_ENCRYPTION_KEY")
                .expect("PB_ENCRYPTION_KEY must be set"),
        };

        let logging = LoggingConfig {
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        };

        let worker = WorkerSettings {
            concurrency: env::var("WORKER_CONCURRENCY")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            poll_interval: env::var("QUEUE_POLL_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            queue_concurrency: env::var("QUEUE_CONCURRENCY")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
        };

        Ok(WorkerConfig {
            database,
            security,
            logging,
            worker,
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

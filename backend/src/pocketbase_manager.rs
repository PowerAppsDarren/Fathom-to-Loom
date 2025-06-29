use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs,
    process::{Child, Command},
    sync::{Mutex, RwLock},
    time::{interval, sleep},
};
use tracing::{error, info, warn, debug};
use serde::{Deserialize, Serialize};

/// Information about a running PocketBase instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocketBaseInstance {
    pub user_id: String,
    pub port: u16,
    pub db_path: PathBuf,
    pub url: String,
    pub status: InstanceStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    Starting,
    Running,
    Failed,
    Stopped,
}

/// Manages lifecycle of per-user PocketBase instances
pub struct PocketBaseManager {
    base_port: u16,
    user_dbs_path: PathBuf,
    binary_path: String,
    instances: Arc<RwLock<HashMap<String, PocketBaseInstance>>>,
    processes: Arc<Mutex<HashMap<String, Child>>>,
    allocated_ports: Arc<Mutex<HashSet<u16>>>,
}

impl PocketBaseManager {
    pub fn new(user_dbs_path: PathBuf, base_port: u16, binary_path: String) -> Self {
        Self {
            base_port,
            user_dbs_path,
            binary_path,
            instances: Arc::new(RwLock::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            allocated_ports: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Initialize a new PocketBase instance for a user
    pub async fn init_user_instance(&self, user_id: &str) -> Result<PocketBaseInstance, PocketBaseError> {
        info!("Initializing PocketBase instance for user: {}", user_id);

        // Check if instance already exists
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(user_id) {
                if instance.status == InstanceStatus::Running {
                    info!("PocketBase instance already running for user: {}", user_id);
                    return Ok(instance.clone());
                }
            }
        }

        // Allocate port
        let port = self.allocate_port(user_id).await?;
        
        // Create database directory and file path
        let db_path = self.user_dbs_path.join(format!("pb_user_{}.db", user_id));
        
        // Ensure user_dbs directory exists
        fs::create_dir_all(&self.user_dbs_path).await
            .map_err(|e| PocketBaseError::IoError(format!("Failed to create user_dbs directory: {}", e)))?;

        let url = format!("http://localhost:{}", port);
        
        let instance = PocketBaseInstance {
            user_id: user_id.to_string(),
            port,
            db_path: db_path.clone(),
            url: url.clone(),
            status: InstanceStatus::Starting,
            created_at: chrono::Utc::now(),
            last_health_check: None,
        };

        // Start PocketBase process
        match self.start_pocketbase_process(&instance).await {
            Ok(child) => {
                // Store process handle
                {
                    let mut processes = self.processes.lock().await;
                    processes.insert(user_id.to_string(), child);
                }

                // Update instance status
                let mut updated_instance = instance.clone();
                updated_instance.status = InstanceStatus::Running;

                // Store instance
                {
                    let mut instances = self.instances.write().await;
                    instances.insert(user_id.to_string(), updated_instance.clone());
                }

                info!("Successfully started PocketBase instance for user {} on port {}", user_id, port);
                Ok(updated_instance)
            }
            Err(e) => {
                // Release the allocated port on failure
                {
                    let mut ports = self.allocated_ports.lock().await;
                    ports.remove(&port);
                }
                
                error!("Failed to start PocketBase instance for user {}: {}", user_id, e);
                Err(e)
            }
        }
    }

    /// Allocate an available port for a user
    async fn allocate_port(&self, user_id: &str) -> Result<u16, PocketBaseError> {
        let mut ports = self.allocated_ports.lock().await;
        
        // Try preferred port first (base_port + user_id hash % 1000)
        let user_hash = self.hash_user_id(user_id);
        let preferred_port = self.base_port + (user_hash % 1000) as u16;
        
        if !ports.contains(&preferred_port) && self.is_port_available(preferred_port).await {
            ports.insert(preferred_port);
            return Ok(preferred_port);
        }

        // Find next available port
        for offset in 1..1000 {
            let port = self.base_port + offset;
            if !ports.contains(&port) && self.is_port_available(port).await {
                ports.insert(port);
                return Ok(port);
            }
        }

        Err(PocketBaseError::NoPortsAvailable)
    }

    /// Hash user ID to get a consistent number
    fn hash_user_id(&self, user_id: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Check if a port is available
    async fn is_port_available(&self, port: u16) -> bool {
        match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Start a PocketBase process for an instance
    async fn start_pocketbase_process(&self, instance: &PocketBaseInstance) -> Result<Child, PocketBaseError> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("serve")
            .arg("--http")
            .arg(format!("127.0.0.1:{}", instance.port))
            .arg("--dir")
            .arg(&instance.db_path.parent().unwrap_or_else(|| std::path::Path::new(".")))
            .env("PB_DATA", &instance.db_path.parent().unwrap_or_else(|| std::path::Path::new(".")))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        debug!("Starting PocketBase with command: {:?}", cmd);

        let child = cmd.spawn()
            .map_err(|e| PocketBaseError::ProcessError(format!("Failed to spawn PocketBase process: {}", e)))?;

        // Give PocketBase time to start
        sleep(Duration::from_secs(2)).await;

        Ok(child)
    }

    /// Start health monitoring for all instances
    pub async fn start_health_monitoring(&self) {
        let instances = Arc::clone(&self.instances);
        let processes = Arc::clone(&self.processes);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
            
            loop {
                interval.tick().await;
                
                let instance_ids: Vec<String> = {
                    let instances_guard = instances.read().await;
                    instances_guard.keys().cloned().collect()
                };

                for user_id in instance_ids {
                    // Check if process is still alive
                    let process_alive = {
                        let mut processes_guard = processes.lock().await;
                        if let Some(child) = processes_guard.get_mut(&user_id) {
                            match child.try_wait() {
                                Ok(None) => true, // Still running
                                Ok(Some(status)) => {
                                    warn!("PocketBase process for user {} exited with status: {:?}", user_id, status);
                                    false
                                }
                                Err(e) => {
                                    error!("Error checking process status for user {}: {}", user_id, e);
                                    false
                                }
                            }
                        } else {
                            false
                        }
                    };

                    // Update instance status and potentially restart
                    {
                        let mut instances_guard = instances.write().await;
                        if let Some(instance) = instances_guard.get_mut(&user_id) {
                            if !process_alive && instance.status == InstanceStatus::Running {
                                warn!("Detected failed PocketBase instance for user: {}", user_id);
                                instance.status = InstanceStatus::Failed;
                                
                                // TODO: Implement auto-restart logic here
                                // For now, just log the failure
                                error!("PocketBase instance for user {} needs restart", user_id);
                            } else if process_alive {
                                // Perform HTTP health check
                                if Self::health_check_http(&instance.url).await {
                                    instance.last_health_check = Some(chrono::Utc::now());
                                    if instance.status != InstanceStatus::Running {
                                        info!("PocketBase instance for user {} is now healthy", user_id);
                                        instance.status = InstanceStatus::Running;
                                    }
                                } else {
                                    warn!("PocketBase instance for user {} failed health check", user_id);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Perform HTTP health check on a PocketBase instance
    async fn health_check_http(url: &str) -> bool {
        let client = reqwest::Client::new();
        let health_url = format!("{}/api/health", url);
        
        match client.get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await 
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get instance information for a user
    pub async fn get_user_instance(&self, user_id: &str) -> Option<PocketBaseInstance> {
        let instances = self.instances.read().await;
        instances.get(user_id).cloned()
    }

    /// Stop a user's PocketBase instance
    pub async fn stop_user_instance(&self, user_id: &str) -> Result<(), PocketBaseError> {
        info!("Stopping PocketBase instance for user: {}", user_id);

        // Kill the process
        {
            let mut processes = self.processes.lock().await;
            if let Some(mut child) = processes.remove(user_id) {
                if let Err(e) = child.kill().await {
                    warn!("Failed to kill PocketBase process for user {}: {}", user_id, e);
                }
            }
        }

        // Update instance status and release port
        {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(user_id) {
                instance.status = InstanceStatus::Stopped;
                
                // Release the port
                let mut ports = self.allocated_ports.lock().await;
                ports.remove(&instance.port);
            }
        }

        Ok(())
    }

    /// Get all running instances
    pub async fn get_all_instances(&self) -> HashMap<String, PocketBaseInstance> {
        let instances = self.instances.read().await;
        instances.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PocketBaseError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Process error: {0}")]
    ProcessError(String),
    
    #[error("No ports available in the allocated range")]
    NoPortsAvailable,
    
    #[error("Instance not found for user: {0}")]
    InstanceNotFound(String),
    
    #[error("Health check failed")]
    HealthCheckFailed,
}

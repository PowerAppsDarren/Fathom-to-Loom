use axum::{extract::FromRef, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{config::Config, pocketbase_manager::PocketBaseManager};
use super::{AppState, queue::Meeting, websocket::WebSocketManager};

/// Enable extracting Config from AppState
impl FromRef<AppState> for Arc<Config> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.clone()
    }
}

/// Enable extracting PocketBaseManager from AppState  
impl FromRef<AppState> for Arc<PocketBaseManager> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.pb_manager.clone()
    }
}

/// Enable extracting WebSocketManager from AppState
impl FromRef<AppState> for Arc<WebSocketManager> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.ws_manager.clone()
    }
}

/// Enable extracting meetings queue from AppState
impl FromRef<AppState> for Arc<RwLock<Vec<Meeting>>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.meetings_queue.clone()
    }
}

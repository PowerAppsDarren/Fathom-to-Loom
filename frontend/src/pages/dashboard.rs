use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::{Route, components::layout::Layout};
use crate::services::{
    auth::AuthService,
    api::{ApiService, Meeting},
    websocket::{WebSocketService, WorkerStatus}
};

#[component]
pub fn Dashboard() -> Element {
    let auth_service = use_context::<Signal<AuthService>>();
    let navigator = use_navigator();
    
    // Redirect if not authenticated
    if !auth_service.read().is_authenticated() {
        navigator.push(Route::Login {});
    }

    let mut queue_data = use_signal(|| Vec::<Meeting>::new());
    let mut worker_status = use_signal(|| Vec::<WorkerStatus>::new());
    let mut is_loading = use_signal(|| true);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut ws_connected = use_signal(|| false);

    // Initialize API service
    let api_service = use_memo(move || {
        ApiService::new(auth_service.read().clone())
    });

    // Load queue data
    use_effect(move || {
        let api = api_service.read().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_queue().await {
                Ok(response) => {
                    if let Some(queue) = response.data {
                        queue_data.set(queue);
                    }
                    is_loading.set(false);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to load queue: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    // Initialize WebSocket connection for real-time updates
    use_effect(move || {
        wasm_bindgen_futures::spawn_local(async move {
            match WebSocketService::new() {
                Ok(mut ws_service) => {
                    match ws_service.connect().await {
                        Ok(()) => {
                            ws_connected.set(true);
                            tracing::info!("WebSocket connected for dashboard updates");
                        }
                        Err(e) => {
                            tracing::error!("Failed to connect WebSocket: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create WebSocket service: {}", e);
                }
            }
        });
    });

    let remove_from_queue = move |meeting_id: uuid::Uuid| {
        let api = api_service.read().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.remove_from_queue(meeting_id).await {
                Ok(response) => {
                    if let Some(updated_queue) = response.data {
                        queue_data.set(updated_queue);
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to remove meeting: {}", e)));
                }
            }
        });
    };

    let calculate_global_position = |queue: &[Meeting], user_id: &str| -> usize {
        queue.iter()
            .position(|m| m.user_id == user_id)
            .map(|pos| pos + 1)
            .unwrap_or(0)
    };

    rsx! {
        Layout {
            div { class: "space-y-6",
                // Header
                div { class: "bg-white shadow rounded-lg p-6",
                    div { class: "flex justify-between items-center",
                        div {
                            h1 { class: "text-2xl font-bold text-gray-900", "Dashboard" }
                            p { class: "text-gray-600 mt-1", "Monitor your queue status and processing progress" }
                        }
                        div { class: "flex items-center space-x-2",
                            if *ws_connected.read() {
                                div { class: "flex items-center text-green-600",
                                    div { class: "w-3 h-3 bg-green-500 rounded-full mr-2 animate-pulse" }
                                    span { class: "text-sm font-medium", "Live Updates" }
                                }
                            } else {
                                div { class: "flex items-center text-gray-500",
                                    div { class: "w-3 h-3 bg-gray-400 rounded-full mr-2" }
                                    span { class: "text-sm", "Disconnected" }
                                }
                            }
                        }
                    }
                }

                if let Some(error) = error_message.read().as_ref() {
                    div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg",
                        "{error}"
                    }
                }

                // Current Queue Status
                div { class: "bg-white shadow rounded-lg p-6",
                    h2 { class: "text-xl font-semibold text-gray-900 mb-4", "Current Queue" }
                    
                    if *is_loading.read() {
                        div { class: "flex justify-center py-8",
                            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                        }
                    } else if queue_data.read().is_empty() {
                        div { class: "text-center py-8",
                            svg { class: "mx-auto h-12 w-12 text-gray-400 mb-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" }
                            }
                            h3 { class: "text-lg font-medium text-gray-900", "No meetings in queue" }
                            p { class: "text-gray-500 mt-1", "Add some recordings to get started!" }
                            Link {
                                to: Route::Recordings {},
                                class: "mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700",
                                "Browse Recordings"
                            }
                        }
                    } else {
                        div { class: "space-y-4",
                            for (index, meeting) in queue_data.read().iter().enumerate() {
                                div { class: "border border-gray-200 rounded-lg p-4",
                                    div { class: "flex justify-between items-start",
                                        div { class: "flex-1",
                                            div { class: "flex items-center space-x-3",
                                                div { class: "flex-shrink-0",
                                                    span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800",
                                                        "#{meeting.position}"
                                                    }
                                                }
                                                h3 { class: "text-lg font-medium text-gray-900", "{meeting.topic}" }
                                            }
                                            p { class: "text-sm text-gray-500 mt-1", "User: {meeting.user_id}" }
                                            
                                            // Progress indicator for current processing
                                            if index == 0 {
                                                div { class: "mt-3",
                                                    div { class: "flex items-center justify-between text-sm",
                                                        span { class: "text-green-600 font-medium", "Currently Processing" }
                                                        span { class: "text-gray-500", "45%" }
                                                    }
                                                    div { class: "mt-1 w-full bg-gray-200 rounded-full h-2",
                                                        div { class: "bg-green-600 h-2 rounded-full transition-all duration-300", style: "width: 45%" }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        div { class: "flex items-center space-x-2",
                                            if let Some(user) = auth_service.read().get_user() {
                                                if meeting.user_id == user.id {
                                                    button {
                                                        class: "text-red-600 hover:text-red-800 text-sm font-medium",
                                                        onclick: move |_| remove_from_queue(meeting.id),
                                                        "Remove"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Global Position & Stats
                if !queue_data.read().is_empty() {
                    if let Some(user) = auth_service.read().get_user() {
                        div { class: "grid md:grid-cols-3 gap-6",
                            div { class: "bg-white shadow rounded-lg p-6",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0",
                                        div { class: "w-8 h-8 bg-indigo-100 rounded-md flex items-center justify-center",
                                            svg { class: "w-5 h-5 text-indigo-600", fill: "currentColor", view_box: "0 0 20 20",
                                                path { d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" }
                                            }
                                        }
                                    }
                                    div { class: "ml-4",
                                        p { class: "text-sm font-medium text-gray-500", "Your Position" }
                                        p { class: "text-2xl font-semibold text-gray-900", 
                                            "{calculate_global_position(&queue_data.read(), &user.id)}"
                                        }
                                    }
                                }
                            }
                            
                            div { class: "bg-white shadow rounded-lg p-6",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0",
                                        div { class: "w-8 h-8 bg-blue-100 rounded-md flex items-center justify-center",
                                            svg { class: "w-5 h-5 text-blue-600", fill: "currentColor", view_box: "0 0 20 20",
                                                path { d: "M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z" }
                                            }
                                        }
                                    }
                                    div { class: "ml-4",
                                        p { class: "text-sm font-medium text-gray-500", "Total in Queue" }
                                        p { class: "text-2xl font-semibold text-gray-900", "{queue_data.read().len()}" }
                                    }
                                }
                            }
                            
                            div { class: "bg-white shadow rounded-lg p-6",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0",
                                        div { class: "w-8 h-8 bg-green-100 rounded-md flex items-center justify-center",
                                            svg { class: "w-5 h-5 text-green-600", fill: "currentColor", view_box: "0 0 20 20",
                                                path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13a1 1 0 102 0V9.414l1.293 1.293a1 1 0 001.414-1.414z" }
                                            }
                                        }
                                    }
                                    div { class: "ml-4",
                                        p { class: "text-sm font-medium text-gray-500", "Est. Wait Time" }
                                        p { class: "text-2xl font-semibold text-gray-900", 
                                            "{(calculate_global_position(&queue_data.read(), &user.id).saturating_sub(1)) * 15}m"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Worker Status
                div { class: "bg-white shadow rounded-lg p-6",
                    h2 { class: "text-xl font-semibold text-gray-900 mb-4", "Worker Status" }
                    
                    if worker_status.read().is_empty() {
                        div { class: "text-center py-6",
                            div { class: "w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center mx-auto mb-4",
                                svg { class: "w-6 h-6 text-gray-400", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" }
                                }
                            }
                            p { class: "text-gray-500", "No active workers" }
                        }
                    } else {
                        div { class: "space-y-4",
                            for worker in worker_status.read().iter() {
                                div { class: "border border-gray-200 rounded-lg p-4",
                                    div { class: "flex justify-between items-center mb-2",
                                        div {
                                            h3 { class: "font-medium text-gray-900", "Worker {worker.worker_id}" }
                                            p { class: "text-sm text-gray-500", "{worker.status}" }
                                        }
                                        span { class: "text-sm font-medium text-gray-900", 
                                            "{(worker.progress * 100.0) as u32}%"
                                        }
                                    }
                                    
                                    div { class: "w-full bg-gray-200 rounded-full h-2 mb-2",
                                        div { 
                                            class: "bg-indigo-600 h-2 rounded-full transition-all duration-300",
                                            style: "width: {worker.progress * 100.0}%"
                                        }
                                    }
                                    
                                    if let Some(task) = &worker.current_task {
                                        p { class: "text-sm text-gray-600", "Current: {task}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

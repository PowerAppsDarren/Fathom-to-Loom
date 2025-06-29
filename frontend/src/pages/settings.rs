use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::{Route, components::layout::Layout};
use crate::services::{
    auth::AuthService,
    api::{ApiService, ApiKey, ApiKeyRequest}
};
use gloo_storage::{LocalStorage, Storage};

#[component]
pub fn Settings() -> Element {
    let auth_service = use_context::<Signal<AuthService>>();
    let navigator = use_navigator();
    
    // Redirect if not authenticated
    if !auth_service.read().is_authenticated() {
        navigator.push(Route::Login {});
    }

    let mut api_keys = use_signal(|| Vec::<ApiKey>::new());
    let mut is_loading = use_signal(|| true);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);

    // Initialize API service
    let api_service = use_memo(move || {
        ApiService::new(auth_service.read().clone())
    });

    // Load API keys
    use_effect(move || {
        let api = api_service.read().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_api_keys().await {
                Ok(keys) => {
                    api_keys.set(keys);
                    is_loading.set(false);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to load API keys: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    let save_api_key = move |name: String, value: String| {
        let api = api_service.read().clone();
        
        let api_key_request = ApiKeyRequest {
            name: name.clone(),
            value: value.clone(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api.save_api_key(api_key_request).await {
                Ok(()) => {
                    success_message.set(Some(format!("API key '{}' saved successfully!", name)));
                    
                    // Reload API keys
                    match api.get_api_keys().await {
                        Ok(keys) => {
                            api_keys.set(keys);
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to reload API keys: {}", e)));
                        }
                    }

                    // Clear success message after 3 seconds
                    gloo_timers::future::TimeoutFuture::new(3000).await;
                    success_message.set(None);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to save API key: {}", e)));
                }
            }
        });
    };

    rsx! {
        Layout {
            div { class: "space-y-6",
                // Header
                div { class: "bg-white shadow rounded-lg p-6",
                    div { class: "flex justify-between items-center",
                        div {
                            h1 { class: "text-2xl font-bold text-gray-900", "Settings" }
                            p { class: "text-gray-600 mt-1", "Manage your API keys and preferences" }
                        }
                    }
                }
            
                // Messages
                if let Some(error) = error_message.read().as_ref() {
                    div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg",
                        "{error}"
                    }
                }

                if let Some(success) = success_message.read().as_ref() {
                    div { class: "bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg",
                        "{success}"
                    }
                }

                // API Keys
                div { class: "bg-white shadow rounded-lg p-6",
                    h2 { class: "text-xl font-semibold text-gray-900 mb-4", "API Keys" }
                    
                    if *is_loading.read() {
                        div { class: "flex justify-center py-8",
                            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                        }
                    } else if api_keys.read().is_empty() {
                        div { class: "text-center py-8",
                            svg { class: "mx-auto h-12 w-12 text-gray-400 mb-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M8 6H5a2 2 0 00-2 2v11a2 2 0 002 2h14a2 2 0 002-2V8a2 2 0 00-2-2h-3M16 2l-4 4m0 0l-4-4m4 4V16" }
                            }
                            h3 { class: "text-lg font-medium text-gray-900", "No API keys found" }
                            p { class: "text-gray-500 mt-1", "Add some API keys to manage your integrations." }
                        }
                    } else {
                        div {
                            for api_key in api_keys.read().iter() {
                                div { class: "border-b border-gray-200 py-4",
                                    div { class: "flex justify-between items-center",
                                        div {
                                            h4 { class: "text-lg font-medium text-gray-900", "{api_key.name}" }
                                            p { class: "text-sm text-gray-500", "Added: {api_key.created_at} UTC" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Add New API Key Form
                    div { class: "mt-6",
                        h3 { class: "text-lg font-medium text-gray-900 mb-4", "Add New API Key" }
                    
                        form {
                            onsubmit: move |evt| {
                                evt.prevent_default();
                                let name = LocalStorage::get("new_api_key_name").unwrap_or_default();
                                let value = LocalStorage::get("new_api_key_value").unwrap_or_default();
                                save_api_key(name, value);
                            },

                            div { class: "grid grid-cols-1 gap-y-4",
                                div {
                                    label { class: "block text-sm font-medium text-gray-700", "API Key Name" }
                                    input {
                                        r#type: "text",
                                        class: "mt-1 block w-full shadow-sm sm:text-sm border border-gray-300 rounded-md",
                                        onchange: move |evt| {
                                            LocalStorage::set("new_api_key_name", evt.value()).unwrap_or_else(|_| ());
                                        }
                                    }
                                }
                                div {
                                    label { class: "block text-sm font-medium text-gray-700", "API Key Value" }
                                    input {
                                        r#type: "text",
                                        class: "mt-1 block w-full shadow-sm sm:text-sm border border-gray-300 rounded-md",
                                        onchange: move |evt| {
                                            LocalStorage::set("new_api_key_value", evt.value()).unwrap_or_else(|_| ());
                                        }
                                    }
                                }
                            }

                            div { class: "pt-5",
                                div { class: "flex justify-end",
                                    button {
                                        r#type: "submit",
                                        class: "bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors",
                                        "Save API Key"
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

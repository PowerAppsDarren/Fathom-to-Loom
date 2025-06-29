mod config;
mod components;
mod pages;
mod services;
mod utils;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use tracing::info;
use config::{load_config, BuildConfig};
use components::{layout::Layout, auth::ProtectedRoute, common::{LoadingSpinner, ErrorMessage, SuccessMessage}};
use pages::*;
use services::{auth::AuthService, websocket::WebSocketService};

// App routes
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/recordings")]
    Recordings {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn RouteComponent() -> Element {
    match use_route::<Route>() {
        Route::Home {} => rsx! { Home {} },
        Route::Login {} => rsx! { Login {} },
        Route::Register {} => rsx! { Register {} },
        Route::Dashboard {} => rsx! { Dashboard {} },
        Route::Recordings {} => rsx! { Recordings {} },
        Route::Settings {} => rsx! { Settings {} },
    }
}

fn main() {
    // Initialize panic hook for better error reporting in development
    console_error_panic_hook::set_once();
    
    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();
    
    info!("Starting Fathom to Loom Frontend v{}", BuildConfig::version());
    info!("Environment: {}", BuildConfig::environment());
    
    if let Some(api_url) = BuildConfig::api_base_url() {
        info!("API Base URL (from build): {}", api_url);
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load configuration on app startup
    let config_future = use_resource(|| async {
        load_config().await
    });

    // Initialize auth service
    let auth_service = use_signal(|| AuthService::new());

    match &*config_future.read_unchecked() {
        Some(Ok(_config)) => {
            // Provide auth service context and router
            use_context_provider(|| auth_service);
            
            rsx! {
                div { class: "min-h-screen bg-gray-50",
                    Router::<Route> {}
                }
            }
        }
        Some(Err(e)) => {
            rsx! {
                div { class: "min-h-screen flex items-center justify-center bg-red-50",
                    div { class: "max-w-md w-full bg-white shadow-lg rounded-lg p-6",
                        div { class: "text-center",
                            h1 { class: "text-2xl font-bold text-red-600 mb-4", "Configuration Error" }
                            p { class: "text-gray-600 mb-4", "Failed to load configuration: {e}" }
                            p { class: "text-sm text-gray-500", "Using default configuration..." }
                        }
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "min-h-screen flex items-center justify-center bg-blue-50",
                    div { class: "text-center",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" }
                        h1 { class: "text-xl font-semibold text-gray-800 mb-2", "Loading..." }
                        p { class: "text-gray-600", "Loading application configuration..." }
                    }
                }
            }
        }
    }
}

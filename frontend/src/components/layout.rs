use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::Route;
use crate::services::auth::AuthService;

#[component]
pub fn Layout(children: Element) -> Element {
    let auth_service = use_context::<Signal<AuthService>>();
    let is_authenticated = auth_service.read().is_authenticated();
    
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            // Navigation bar
            nav { class: "bg-white shadow-sm border-b border-gray-200",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div { class: "flex justify-between h-16",
                        div { class: "flex items-center",
                            Link {
                                to: Route::Home {},
                                class: "text-xl font-bold text-indigo-600 hover:text-indigo-800 transition-colors",
                                "Fathom to Loom"
                            }
                        }
                        
                        // Navigation links
                        if is_authenticated {
                            div { class: "flex items-center space-x-4",
                                Link {
                                    to: Route::Dashboard {},
                                    class: "text-gray-700 hover:text-indigo-600 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                                    "Dashboard"
                                }
                                Link {
                                    to: Route::Recordings {},
                                    class: "text-gray-700 hover:text-indigo-600 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                                    "Recordings"
                                }
                                Link {
                                    to: Route::Settings {},
                                    class: "text-gray-700 hover:text-indigo-600 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                                    "Settings"
                                }
                                button {
                                    class: "bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors",
                                    onclick: move |_| {
                                        auth_service.write().logout();
                                    },
                                    "Logout"
                                }
                            }
                        } else {
                            div { class: "flex items-center space-x-4",
                                Link {
                                    to: Route::Login {},
                                    class: "text-gray-700 hover:text-indigo-600 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                                    "Login"
                                }
                                Link {
                                    to: Route::Register {},
                                    class: "bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors",
                                    "Register"
                                }
                            }
                        }
                    }
                }
            }
            
            // Main content
            main { class: "max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8",
                {children}
            }
        }
    }
}

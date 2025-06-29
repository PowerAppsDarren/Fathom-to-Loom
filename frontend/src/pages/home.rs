use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::{Route, components::layout::Layout};
use crate::services::auth::AuthService;

#[component]
pub fn Home() -> Element {
    let auth_service = use_context::<Signal<AuthService>>();
    let is_authenticated = auth_service.read().is_authenticated();
    
    rsx! {
        Layout {
            div { class: "text-center",
                div { class: "max-w-4xl mx-auto",
                    // Hero section
                    div { class: "mb-12",
                        h1 { class: "text-4xl font-bold text-gray-900 mb-4",
                            "Fathom to Loom"
                        }
                        p { class: "text-xl text-gray-600 mb-8",
                            "Transform your Fathom meeting recordings into polished Loom videos with AI-powered processing."
                        }
                        
                        if !is_authenticated {
                            div { class: "space-x-4",
                                Link {
                                    to: Route::Register {},
                                    class: "bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-3 px-6 rounded-lg transition-colors",
                                    "Get Started"
                                }
                                Link {
                                    to: Route::Login {},
                                    class: "bg-gray-200 hover:bg-gray-300 text-gray-800 font-bold py-3 px-6 rounded-lg transition-colors",
                                    "Sign In"
                                }
                            }
                        } else {
                            div { class: "space-x-4",
                                Link {
                                    to: Route::Dashboard {},
                                    class: "bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-3 px-6 rounded-lg transition-colors",
                                    "Go to Dashboard"
                                }
                                Link {
                                    to: Route::Recordings {},
                                    class: "bg-gray-200 hover:bg-gray-300 text-gray-800 font-bold py-3 px-6 rounded-lg transition-colors",
                                    "View Recordings"
                                }
                            }
                        }
                    }
                    
                    // Features section
                    div { class: "grid md:grid-cols-3 gap-8 text-left",
                        div { class: "bg-white p-6 rounded-lg shadow-md",
                            div { class: "w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-indigo-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" }
                                }
                            }
                            h3 { class: "text-lg font-semibold text-gray-900 mb-2", "Import Recordings" }
                            p { class: "text-gray-600", "Connect your Fathom account and automatically import meeting recordings for processing." }
                        }
                        
                        div { class: "bg-white p-6 rounded-lg shadow-md",
                            div { class: "w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-indigo-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" }
                                }
                            }
                            h3 { class: "text-lg font-semibold text-gray-900 mb-2", "AI Processing" }
                            p { class: "text-gray-600", "Our AI enhances audio quality, removes filler words, and optimizes your content for professional presentation." }
                        }
                        
                        div { class: "bg-white p-6 rounded-lg shadow-md",
                            div { class: "w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-indigo-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" }
                                }
                            }
                            h3 { class: "text-lg font-semibold text-gray-900 mb-2", "Export to Loom" }
                            p { class: "text-gray-600", "Seamlessly upload your processed videos to Loom with professional quality and formatting." }
                        }
                    }
                }
            }
        }
    }
}

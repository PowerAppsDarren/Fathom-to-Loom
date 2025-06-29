use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::{Route, components::layout::Layout};
use crate::services::{
    auth::AuthService,
    api::{ApiService, FathomMeeting, MeetingRequest}
};

#[component]
pub fn Recordings() -> Element {
    let auth_service = use_context::<Signal<AuthService>>();
    let navigator = use_navigator();
    
    // Redirect if not authenticated
    if !auth_service.read().is_authenticated() {
        navigator.push(Route::Login {});
    }

    let mut meetings_data = use_signal(|| Vec::<FathomMeeting>::new());
    let mut is_loading = use_signal(|| true);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);
    let mut adding_to_queue = use_signal(|| std::collections::HashSet::<String>::new());

    // Initialize API service
    let api_service = use_memo(move || {
        ApiService::new(auth_service.read().clone())
    });

    // Load meetings data
    use_effect(move || {
        let api = api_service.read().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_meetings(Some(50), None).await {
                Ok(response) => {
                    meetings_data.set(response.meetings);
                    is_loading.set(false);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to load meetings: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    let add_to_queue = move |meeting: FathomMeeting| {
        let api = api_service.read().clone();
        let meeting_id = meeting.id.clone();
        
        // Add to loading set
        adding_to_queue.write().insert(meeting_id.clone());
        
        if let Some(user) = auth_service.read().get_user() {
            let meeting_request = MeetingRequest {
                user_id: user.id.clone(),
                topic: meeting.title.clone(),
            };

            wasm_bindgen_futures::spawn_local(async move {
                match api.add_to_queue(meeting_request).await {
                    Ok(_response) => {
                        success_message.set(Some(format!("'{}' added to queue successfully!", meeting.title)));
                        adding_to_queue.write().remove(&meeting_id);
                        
                        // Clear success message after 3 seconds
                        gloo_timers::future::TimeoutFuture::new(3000).await;
                        success_message.set(None);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to add to queue: {}", e)));
                        adding_to_queue.write().remove(&meeting_id);
                    }
                }
            });
        }
    };

    let format_duration = |duration_seconds: u32| -> String {
        let hours = duration_seconds / 3600;
        let minutes = (duration_seconds % 3600) / 60;
        
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    };

    let format_date = |date_str: &str| -> String {
        // Simple date formatting - in a real app you'd use chrono
        date_str.split('T')
            .next()
            .unwrap_or(date_str)
            .to_string()
    };

    rsx! {
        Layout {
            div { class: "space-y-6",
                // Header
                div { class: "bg-white shadow rounded-lg p-6",
                    div { class: "flex justify-between items-center",
                        div {
                            h1 { class: "text-2xl font-bold text-gray-900", "Fathom Recordings" }
                            p { class: "text-gray-600 mt-1", "Browse and add your meeting recordings to the processing queue" }
                        }
                        div { class: "flex items-center space-x-4",
                            Link {
                                to: Route::Dashboard {},
                                class: "bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors",
                                "View Queue"
                            }
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

                // Meetings list
                div { class: "bg-white shadow rounded-lg",
                    div { class: "px-6 py-4 border-b border-gray-200",
                        h2 { class: "text-lg font-semibold text-gray-900", "Available Recordings" }
                    }
                    
                    if *is_loading.read() {
                        div { class: "flex justify-center py-12",
                            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                        }
                    } else if meetings_data.read().is_empty() {
                        div { class: "text-center py-12",
                            svg { class: "mx-auto h-12 w-12 text-gray-400 mb-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" }
                            }
                            h3 { class: "text-lg font-medium text-gray-900", "No recordings found" }
                            p { class: "text-gray-500 mt-2", "Check your Fathom API key settings or record some meetings first." }
                            Link {
                                to: Route::Settings {},
                                class: "mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700",
                                "Configure API Keys"
                            }
                        }
                    } else {
                        div { class: "divide-y divide-gray-200",
                            for meeting in meetings_data.read().iter() {
                                div { class: "p-6 hover:bg-gray-50 transition-colors",
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex-1 min-w-0",
                                            div { class: "flex items-start justify-between",
                                                div { class: "flex-1",
                                                    h3 { class: "text-lg font-medium text-gray-900 mb-1",
                                                        "{meeting.title}"
                                                    }
                                                    div { class: "flex items-center space-x-4 text-sm text-gray-500 mb-2",
                                                        div { class: "flex items-center",
                                                            svg { class: "w-4 h-4 mr-1", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" }
                                                            }
                                                            "{format_date(&meeting.start_time)}"
                                                        }
                                                        div { class: "flex items-center",
                                                            svg { class: "w-4 h-4 mr-1", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" }
                                                            }
                                                            "{format_duration(meeting.duration)}"
                                                        }
                                                        div { class: "flex items-center",
                                                            svg { class: "w-4 h-4 mr-1", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" }
                                                            }
                                                            "{meeting.participants.len()} participants"
                                                        }
                                                    }
                                                    
                                                    if !meeting.participants.is_empty() {
                                                        div { class: "flex flex-wrap gap-1 mt-2",
                                                            for participant in meeting.participants.iter().take(5) {
                                                                span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800",
                                                                    "{participant}"
                                                                }
                                                            }
                                                            if meeting.participants.len() > 5 {
                                                                span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800",
                                                                    "+{} more", meeting.participants.len() - 5
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        div { class: "flex items-center space-x-3 ml-4",
                                            if adding_to_queue.read().contains(&meeting.id) {
                                                button {
                                                    class: "bg-gray-300 text-gray-500 px-4 py-2 rounded-md text-sm font-medium cursor-not-allowed",
                                                    disabled: true,
                                                    div { class: "flex items-center",
                                                        div { class: "animate-spin rounded-full h-4 w-4 border-b-2 border-gray-500 mr-2" }
                                                        "Adding..."
                                                    }
                                                }
                                            } else {
                                                button {
                                                    class: "bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors",
                                                    onclick: {
                                                        let meeting_clone = meeting.clone();
                                                        move |_| {
                                                            add_to_queue(meeting_clone.clone());
                                                        }
                                                    },
                                                    "Add to Queue"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Instructions
                div { class: "bg-blue-50 border border-blue-200 rounded-lg p-6",
                    div { class: "flex",
                        div { class: "flex-shrink-0",
                            svg { class: "h-5 w-5 text-blue-400", fill: "currentColor", view_box: "0 0 20 20",
                                path { fill_rule: "evenodd", d: "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z", clip_rule: "evenodd" }
                            }
                        }
                        div { class: "ml-3",
                            h3 { class: "text-sm font-medium text-blue-800", "How it works" }
                            div { class: "mt-2 text-sm text-blue-700",
                                ul { class: "list-disc list-inside space-y-1",
                                    li { "Select recordings from your Fathom account and add them to the processing queue" }
                                    li { "Our AI will enhance the audio quality and remove filler words" }
                                    li { "Processed videos will be automatically uploaded to your Loom account" }
                                    li { "Monitor progress in real-time on the " 
                                        Link {
                                            to: Route::Dashboard {},
                                            class: "underline font-medium",
                                            "Dashboard"
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
}

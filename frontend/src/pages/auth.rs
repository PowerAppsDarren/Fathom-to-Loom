use dioxus::prelude::*;
use dioxus_router::prelude::*;
use validator::Validate;
use crate::{Route, components::layout::Layout};
use crate::services::auth::{AuthService, LoginRequest, RegisterRequest};

#[derive(Debug, Clone, Validate)]
struct LoginForm {
    #[validate(email(message = "Please enter a valid email address"))]
    email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    password: String,
}

#[derive(Debug, Clone, Validate)]
struct RegisterForm {
    #[validate(email(message = "Please enter a valid email address"))]
    email: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    username: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    password: String,
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    password_confirm: String,
}

#[component]
pub fn Login() -> Element {
    let mut auth_service = use_context::<Signal<AuthService>>();
    let navigator = use_navigator();
    
    let mut form_data = use_signal(|| LoginForm {
        email: String::new(),
        password: String::new(),
    });
    
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    // Redirect if already authenticated
    if auth_service.read().is_authenticated() {
        navigator.push(Route::Dashboard {});
    }

    let handle_submit = move |_evt: FormEvent| {
        let form = form_data.read();
        
        // Validate form
        if let Err(validation_errors) = form.validate() {
            let errors: Vec<String> = validation_errors
                .field_errors()
                .values()
                .flat_map(|errors| errors.iter().map(|e| e.message.as_ref().unwrap_or(&std::borrow::Cow::Borrowed("Validation error")).to_string()))
                .collect();
            error_message.set(Some(errors.join(", ")));
            return;
        }

        is_loading.set(true);
        error_message.set(None);

        let login_request = LoginRequest {
            email: form.email.clone(),
            password: form.password.clone(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match auth_service.write().login(login_request).await {
                Ok(()) => {
                    navigator.push(Route::Dashboard {});
                }
                Err(e) => {
                    error_message.set(Some(format!("Login failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        Layout {
            div { class: "max-w-md mx-auto mt-8",
                div { class: "bg-white shadow-lg rounded-lg p-6",
                    h2 { class: "text-2xl font-bold text-gray-900 mb-6 text-center", "Sign In" }
                    
                    if let Some(error) = error_message.read().as_ref() {
                        div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4",
                            "{error}"
                        }
                    }

                    form { onsubmit: handle_submit,
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "email",
                                "Email"
                            }
                            input {
                                r#type: "email",
                                id: "email",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().email}",
                                oninput: move |evt| {
                                    form_data.write().email = evt.value();
                                },
                                required: true
                            }
                        }

                        div { class: "mb-6",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "password",
                                "Password"
                            }
                            input {
                                r#type: "password",
                                id: "password",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().password}",
                                oninput: move |evt| {
                                    form_data.write().password = evt.value();
                                },
                                required: true
                            }
                        }

                        button {
                            r#type: "submit",
                            class: "w-full bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded-md transition-colors",
                            disabled: *is_loading.read(),
                            if *is_loading.read() { "Signing In..." } else { "Sign In" }
                        }
                    }

                    div { class: "mt-6 text-center",
                        p { class: "text-sm text-gray-600",
                            "Don't have an account? "
                            Link {
                                to: Route::Register {},
                                class: "text-indigo-600 hover:text-indigo-800 font-medium",
                                "Sign up"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let mut auth_service = use_context::<Signal<AuthService>>();
    let navigator = use_navigator();
    
    let mut form_data = use_signal(|| RegisterForm {
        email: String::new(),
        username: String::new(),
        password: String::new(),
        password_confirm: String::new(),
    });
    
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    // Redirect if already authenticated
    if auth_service.read().is_authenticated() {
        navigator.push(Route::Dashboard {});
    }

    let handle_submit = move |_evt: FormEvent| {
        let form = form_data.read();
        
        // Validate form
        if let Err(validation_errors) = form.validate() {
            let errors: Vec<String> = validation_errors
                .field_errors()
                .values()
                .flat_map(|errors| errors.iter().map(|e| e.message.as_ref().unwrap_or(&std::borrow::Cow::Borrowed("Validation error")).to_string()))
                .collect();
            error_message.set(Some(errors.join(", ")));
            return;
        }

        is_loading.set(true);
        error_message.set(None);

        let register_request = RegisterRequest {
            email: form.email.clone(),
            username: form.username.clone(),
            password: form.password.clone(),
            password_confirm: form.password_confirm.clone(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match auth_service.write().register(register_request).await {
                Ok(()) => {
                    navigator.push(Route::Dashboard {});
                }
                Err(e) => {
                    error_message.set(Some(format!("Registration failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        Layout {
            div { class: "max-w-md mx-auto mt-8",
                div { class: "bg-white shadow-lg rounded-lg p-6",
                    h2 { class: "text-2xl font-bold text-gray-900 mb-6 text-center", "Create Account" }
                    
                    if let Some(error) = error_message.read().as_ref() {
                        div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4",
                            "{error}"
                        }
                    }

                    form { onsubmit: handle_submit,
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "email",
                                "Email"
                            }
                            input {
                                r#type: "email",
                                id: "email",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().email}",
                                oninput: move |evt| {
                                    form_data.write().email = evt.value();
                                },
                                required: true
                            }
                        }

                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "username",
                                "Username"
                            }
                            input {
                                r#type: "text",
                                id: "username",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().username}",
                                oninput: move |evt| {
                                    form_data.write().username = evt.value();
                                },
                                required: true
                            }
                        }

                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "password",
                                "Password"
                            }
                            input {
                                r#type: "password",
                                id: "password",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().password}",
                                oninput: move |evt| {
                                    form_data.write().password = evt.value();
                                },
                                required: true
                            }
                        }

                        div { class: "mb-6",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", r#for: "password_confirm",
                                "Confirm Password"
                            }
                            input {
                                r#type: "password",
                                id: "password_confirm",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                value: "{form_data.read().password_confirm}",
                                oninput: move |evt| {
                                    form_data.write().password_confirm = evt.value();
                                },
                                required: true
                            }
                        }

                        button {
                            r#type: "submit",
                            class: "w-full bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded-md transition-colors",
                            disabled: *is_loading.read(),
                            if *is_loading.read() { "Creating Account..." } else { "Create Account" }
                        }
                    }

                    div { class: "mt-6 text-center",
                        p { class: "text-sm text-gray-600",
                            "Already have an account? "
                            Link {
                                to: Route::Login {},
                                class: "text-indigo-600 hover:text-indigo-800 font-medium",
                                "Sign in"
                            }
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
    }
}

#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg",
            "{message}"
        }
    }
}

#[component]
pub fn SuccessMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg",
            "{message}"
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn ProgressBar(progress: f32, color: Option<String>) -> Element {
    let color_class = color.unwrap_or_else(|| "bg-indigo-600".to_string());
    
    rsx! {
        div { class: "w-full bg-gray-200 rounded-full h-2",
            div { 
                class: "{color_class} h-2 rounded-full transition-all duration-300",
                style: "width: {progress * 100.0}%"
            }
        }
    }
}

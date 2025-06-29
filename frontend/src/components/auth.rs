use dioxus::prelude::*;

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    rsx! {
        {children}
    }
}

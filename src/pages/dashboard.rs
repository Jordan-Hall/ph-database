use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Dashboard" }
                p { class: "govuk-body", "Welcome to your dashboard." }
            }
        }
    }
}

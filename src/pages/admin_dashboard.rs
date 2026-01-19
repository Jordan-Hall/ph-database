use dioxus::prelude::*;

#[component]
pub fn AdminDashboard() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Admin Dashboard" }
            }
        }
    }
}

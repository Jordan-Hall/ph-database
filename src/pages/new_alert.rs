use dioxus::prelude::*;

#[component]
pub fn NewAlert() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "New Alert" }
            }
        }
    }
}

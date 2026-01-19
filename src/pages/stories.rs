use dioxus::prelude::*;

#[component]
pub fn Stories() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Survivor Stories" }
            }
        }
    }
}

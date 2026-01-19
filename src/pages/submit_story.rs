use dioxus::prelude::*;

#[component]
pub fn SubmitStory() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Submit Your Story" }
            }
        }
    }
}

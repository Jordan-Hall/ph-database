use dioxus::prelude::*;

#[component]
pub fn PublishedItem(slug: String) -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Published Item: {slug}" }
            }
        }
    }
}

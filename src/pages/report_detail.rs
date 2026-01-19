use dioxus::prelude::*;

#[component]
pub fn ReportDetail(id: String) -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Report: {id}" }
            }
        }
    }
}

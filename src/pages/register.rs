use dioxus::prelude::*;

#[component]
pub fn Register() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Register - Coming Soon" }
                p { class: "govuk-body",
                    "Registration page will be implemented here."
                }
            }
        }
    }
}

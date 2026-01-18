use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "govuk-header",
            div { class: "govuk-header__container",
                div { class: "govuk-header__logo",
                    "GOV.UK"
                }
                div { class: "govuk-header__product-name",
                    "Predator Hunters Database"
                }
            }
        }
        div { class: "govuk-phase-banner",
            div { class: "govuk-phase-banner__content",
                strong { class: "govuk-tag", "BETA" }
                span { class: "govuk-phase-banner__text",
                    "This is a journalism database for tracking criminal convictions from court records and interviews."
                }
            }
        }
    }
}

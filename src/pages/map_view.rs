use dioxus::prelude::*;

#[component]
pub fn MapView() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Map View" }
                div { class: "govuk-inset-text",
                    p { "Interactive map integration with MapLibre GL will be displayed here." }
                    p { "Map features:" }
                    ul { class: "govuk-list govuk-list--bullet",
                        li { "Geo-bounded queries for incident locations" }
                        li { "Privacy-preserving fuzzy display" }
                        li { "Multiple precision classes (exact, street, district, city)" }
                        li { "Harm risk visualization" }
                    }
                }
            }
        }
    }
}

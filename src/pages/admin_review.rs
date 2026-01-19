use dioxus::prelude::*;

#[component]
pub fn AdminReview() -> Element {
    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Review Queue" }

                Link { to: "/admin", class: "govuk-back-link", "Back to admin dashboard" }

                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                div { class: "govuk-inset-text",
                    p { "Review queue functionality will display pending items for review:" }
                    ul { class: "govuk-list govuk-list--bullet",
                        li { "Reports awaiting approval" }
                        li { "Survivor stories for review" }
                        li { "Map entries pending verification" }
                        li { "Content corrections and takedown requests" }
                    }
                }

                div { class: "govuk-notification-banner", role: "region",
                    div { class: "govuk-notification-banner__header",
                        h2 { class: "govuk-notification-banner__title", "No items pending review" }
                    }
                    div { class: "govuk-notification-banner__content",
                        p { class: "govuk-notification-banner__heading", "All review items have been processed." }
                    }
                }
            }
        }
    }
}

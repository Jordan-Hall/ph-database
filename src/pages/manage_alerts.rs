use dioxus::prelude::*;
use crate::api::{ApiClient, Alert};

#[component]
pub fn ManageAlerts() -> Element {
    let mut alerts = use_signal(|| Vec::<Alert>::new());
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            if let Ok(data) = api.get_active_alerts().await {
                alerts.set(data);
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Manage Alerts" }

                div { class: "govuk-button-group",
                    Link { to: "/alerts/new", class: "govuk-button", "Create new alert" }
                    Link { to: "/alerts", class: "govuk-button govuk-button--secondary", "View public alerts" }
                }

                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                if loading() {
                    p { class: "govuk-body", "Loading alerts..." }
                } else if alerts().is_empty() {
                    div { class: "govuk-inset-text",
                        p { "No alerts found. ", Link { to: "/alerts/new", "Create one" } }
                    }
                } else {
                    for alert in alerts() {
                        div { class: "govuk-summary-card",
                            div { class: "govuk-summary-card__title-wrapper",
                                h2 { class: "govuk-summary-card__title", "{alert.full_name}" }
                            }
                            div { class: "govuk-summary-card__content",
                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Status" }
                                        dd { class: "govuk-summary-list__value",
                                            strong { class: "govuk-tag", "{alert.status}" }
                                        }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Last Seen" }
                                        dd { class: "govuk-summary-list__value", "{alert.last_seen_location}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

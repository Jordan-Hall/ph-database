use dioxus::prelude::*;
use crate::api::{ApiClient, Alert};

#[component]
pub fn PublicAlerts() -> Element {
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
                h1 { class: "govuk-heading-xl", "Active Missing Person Alerts" }

                if loading() {
                    p { class: "govuk-body", "Loading alerts..." }
                } else if alerts().is_empty() {
                    div { class: "govuk-inset-text",
                        p { "There are currently no active alerts." }
                    }
                } else {
                    for alert in alerts() {
                        div { class: "govuk-summary-card",
                            div { class: "govuk-summary-card__title-wrapper",
                                h2 { class: "govuk-summary-card__title", "{alert.full_name}" }
                                ul { class: "govuk-summary-card__actions",
                                    li { class: "govuk-summary-card__action",
                                        strong {
                                            class: match alert.priority.as_str() {
                                                "urgent" => "govuk-tag govuk-tag--red",
                                                "high" => "govuk-tag govuk-tag--orange",
                                                _ => "govuk-tag govuk-tag--yellow",
                                            },
                                            "{alert.priority}"
                                        }
                                    }
                                }
                            }
                            div { class: "govuk-summary-card__content",
                                dl { class: "govuk-summary-list",
                                    if let Some(age) = alert.age {
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Age" }
                                            dd { class: "govuk-summary-list__value", "{age}" }
                                        }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Last Seen" }
                                        dd { class: "govuk-summary-list__value", "{alert.last_seen_location}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Description" }
                                        dd { class: "govuk-summary-list__value", "{alert.description}" }
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

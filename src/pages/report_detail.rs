use dioxus::prelude::*;
use dioxus_router::Link;
use crate::api::{ApiClient, Report};

#[component]
pub fn ReportDetail(id: String) -> Element {
    let mut report = use_signal(|| None::<Report>);
    let mut loading = use_signal(|| true);

    // Load report
    use_effect(move || {
        let report_id = id.clone();
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.get_report(&report_id).await {
                Ok(data) => report.set(Some(data)),
                Err(e) => log::error!("Failed to load report: {}", e),
            }
            
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                if loading() {
                    p { class: "govuk-body", "Loading report..." }
                } else if let Some(r) = report() {
                    div {
                        h1 { class: "govuk-heading-xl", "{r.title}" }

                        div { class: "govuk-grid-row",
                            div { class: "govuk-grid-column-two-thirds",
                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Subject" }
                                        dd { class: "govuk-summary-list__value", "{r.subject_name}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Offense Type" }
                                        dd { class: "govuk-summary-list__value", "{r.offense_type}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Status" }
                                        dd { class: "govuk-summary-list__value",
                                            strong { class: "govuk-tag", "{r.status}" }
                                        }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Harm Risk" }
                                        dd { class: "govuk-summary-list__value",
                                            strong {
                                                class: match r.harm_risk.as_str() {
                                                    "high" => "govuk-tag govuk-tag--red",
                                                    "medium" => "govuk-tag govuk-tag--yellow",
                                                    _ => "govuk-tag govuk-tag--grey",
                                                },
                                                "{r.harm_risk}"
                                            }
                                        }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Visibility" }
                                        dd { class: "govuk-summary-list__value", "{r.visibility}" }
                                    }
                                }

                                h2 { class: "govuk-heading-m", "Description" }
                                p { class: "govuk-body", "{r.description}" }

                                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                                Link { to: "/reports", class: "govuk-button govuk-button--secondary", "Back to reports" }
                            }
                        }
                    }
                } else {
                    div { class: "govuk-error-summary",
                        h2 { class: "govuk-error-summary__title", "Report not found" }
                        div { class: "govuk-error-summary__body",
                            p { "The report you're looking for doesn't exist or you don't have permission to view it." }
                        }
                        Link { to: "/reports", class: "govuk-link", "Back to reports" }
                    }
                }
            }
        }
    }
}

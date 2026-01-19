use dioxus::prelude::*;
use crate::api::{ApiClient, Report};
use crate::notifications::NotificationService;

#[component]
pub fn Reports() -> Element {
    let mut reports = use_signal(|| Vec::<Report>::new());
    let mut loading = use_signal(|| true);
    let mut search_query = use_signal(|| String::new());

    // Load reports
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.get_reports().await {
                Ok(data) => reports.set(data),
                Err(e) => log::error!("Failed to load reports: {}", e),
            }
            
            loading.set(false);
        });
    });

    let filtered_reports = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            reports()
        } else {
            reports()
                .into_iter()
                .filter(|r| {
                    r.title.to_lowercase().contains(&query)
                        || r.subject_name.to_lowercase().contains(&query)
                        || r.offense_type.to_lowercase().contains(&query)
                })
                .collect()
        }
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Reports" }

                div { class: "govuk-grid-row",
                    div { class: "govuk-grid-column-two-thirds",
                        div { class: "govuk-form-group",
                            label { class: "govuk-label", r#for: "search", "Search reports" }
                            input {
                                class: "govuk-input",
                                id: "search",
                                name: "search",
                                r#type: "text",
                                placeholder: "Search by title, subject, or offense type...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value()),
                            }
                        }
                    }
                    div { class: "govuk-grid-column-one-third",
                        Link {
                            to: "/reports/new",
                            class: "govuk-button",
                            style: "margin-top: 30px;",
                            "Create new report"
                        }
                    }
                }

                if loading() {
                    p { class: "govuk-body", "Loading reports..." }
                } else {
                    if filtered_reports().is_empty() {
                        div { class: "govuk-inset-text",
                            p { "No reports found. ", Link { to: "/reports/new", "Create the first report" } }
                        }
                    } else {
                        p { class: "govuk-body", "Showing {filtered_reports().len()} report(s)" }
                        
                        for report in filtered_reports() {
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h2 { class: "govuk-summary-card__title",
                                        Link {
                                            to: format!("/reports/{}", report.id.as_ref().unwrap_or(&"unknown".to_string())),
                                            "{report.title}"
                                        }
                                    }
                                    ul { class: "govuk-summary-card__actions",
                                        li { class: "govuk-summary-card__action",
                                            Link {
                                                to: format!("/reports/{}", report.id.as_ref().unwrap_or(&"unknown".to_string())),
                                                class: "govuk-link",
                                                "View details"
                                            }
                                        }
                                    }
                                }
                                div { class: "govuk-summary-card__content",
                                    dl { class: "govuk-summary-list",
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Subject" }
                                            dd { class: "govuk-summary-list__value", "{report.subject_name}" }
                                        }
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Offense Type" }
                                            dd { class: "govuk-summary-list__value", "{report.offense_type}" }
                                        }
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Status" }
                                            dd { class: "govuk-summary-list__value",
                                                strong { class: "govuk-tag", "{report.status}" }
                                            }
                                        }
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Harm Risk" }
                                            dd { class: "govuk-summary-list__value",
                                                strong {
                                                    class: match report.harm_risk.as_str() {
                                                        "high" => "govuk-tag govuk-tag--red",
                                                        "medium" => "govuk-tag govuk-tag--yellow",
                                                        _ => "govuk-tag govuk-tag--grey",
                                                    },
                                                    "{report.harm_risk}"
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
        }
    }
}

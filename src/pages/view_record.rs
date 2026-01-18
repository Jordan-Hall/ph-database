use dioxus::prelude::*;
use dioxus_router::{Link, hooks::use_navigator};
use crate::database::Database;
use crate::models::{ConvictionRecord, RecordSource};
use crate::components::InteractiveMap;

#[derive(Clone, PartialEq, Props)]
pub struct ViewRecordProps {
    id: String,
}

#[component]
pub fn ViewRecord(props: ViewRecordProps) -> Element {
    let nav = use_navigator();
    let mut record = use_signal(|| None::<ConvictionRecord>);
    let mut delete_status = use_signal(|| None::<String>);
    let id = props.id.clone();

    use_effect(move || {
        let id = id.clone();
        spawn(async move {
            let db = Database::new().unwrap();
            match db.get_record(&id).await {
                Ok(Some(r)) => record.set(Some(r)),
                Ok(None) => log::error!("Record not found"),
                Err(e) => log::error!("Error loading record: {}", e),
            }
        });
    });

    let delete_record = move |_| {
        let id = props.id.clone();
        spawn(async move {
            let db = Database::new().unwrap();
            match db.delete_record(&id).await {
                Ok(_) => {
                    delete_status.set(Some("Record deleted successfully".to_string()));
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    nav.push("/");
                }
                Err(e) => {
                    delete_status.set(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                Link { to: "/", class: "govuk-link", "← Back to home" }

                if let Some(r) = record() {
                    div {
                        h1 { class: "govuk-heading-xl", "{r.full_name}" }

                        if let Some(status) = delete_status() {
                            div {
                                class: "result-card",
                                style: if status.contains("Error") {
                                    "border-color: #d4351c; background-color: #f8d7da;"
                                } else {
                                    "border-color: #00703c; background-color: #d4edda;"
                                },
                                p { class: "govuk-body", "{status}" }
                            }
                        }

                        div { class: "govuk-grid-row",
                            div { class: "govuk-grid-column-two-thirds",
                                h2 { class: "govuk-heading-l", "Conviction Details" }

                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Offense Type" }
                                        dd { class: "govuk-summary-list__value", "{r.offense_type}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Conviction Date" }
                                        dd { class: "govuk-summary-list__value", "{r.conviction_date}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Court" }
                                        dd { class: "govuk-summary-list__value", "{r.court_name}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Sentence" }
                                        dd { class: "govuk-summary-list__value", "{r.sentence}" }
                                    }
                                }

                                h2 { class: "govuk-heading-l", "Location" }

                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Street" }
                                        dd { class: "govuk-summary-list__value", "{r.street_name}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "City" }
                                        dd { class: "govuk-summary-list__value", "{r.city}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Postcode District" }
                                        dd { class: "govuk-summary-list__value", "{r.postcode_district}" }
                                    }
                                }

                                h2 { class: "govuk-heading-l", "Source Information" }

                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Source" }
                                        dd { class: "govuk-summary-list__value",
                                            match r.source {
                                                RecordSource::CourtRecord => "Court Record",
                                                RecordSource::Interview => "Interview",
                                                RecordSource::Both => "Court Record & Interview",
                                            }
                                        }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Interview Consent" }
                                        dd { class: "govuk-summary-list__value",
                                            if r.interview_consent { "Yes" } else { "No" }
                                        }
                                    }
                                    if let Some(interview_date) = &r.interview_date {
                                        div { class: "govuk-summary-list__row",
                                            dt { class: "govuk-summary-list__key", "Interview Date" }
                                            dd { class: "govuk-summary-list__value", "{interview_date}" }
                                        }
                                    }
                                }

                                if !r.notes.is_empty() {
                                    h2 { class: "govuk-heading-l", "Additional Notes" }
                                    p { class: "govuk-body", "{r.notes}" }
                                }

                                div { class: "govuk-!-margin-top-6",
                                    button {
                                        class: "govuk-button govuk-button--warning",
                                        onclick: delete_record,
                                        "Delete Record"
                                    }
                                }
                            }

                            div { class: "govuk-grid-column-one-third",
                                h2 { class: "govuk-heading-m", "Location on Map" }
                                if r.latitude.is_some() && r.longitude.is_some() {
                                    InteractiveMap { records: vec![r.clone()] }
                                } else {
                                    p { class: "govuk-body govuk-hint",
                                        "No coordinates available for this record."
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div {
                        h1 { class: "govuk-heading-xl", "Loading..." }
                        p { class: "govuk-body", "Please wait while we load the record." }
                    }
                }
            }
        }
    }
}

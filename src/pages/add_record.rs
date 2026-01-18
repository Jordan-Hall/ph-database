use dioxus::prelude::*;
use dioxus_router::{Link, hooks::use_navigator};
use crate::components::*;
use crate::database::Database;
use crate::models::{ConvictionRecord, RecordSource};

#[component]
pub fn AddRecord() -> Element {
    let nav = use_navigator();
    let mut record = use_signal(|| ConvictionRecord::new());
    let mut save_status = use_signal(|| None::<String>);

    let save_record = move |_| {
        spawn(async move {
            let db = Database::new().unwrap();
            match db.create_record(record()).await {
                Ok(_) => {
                    save_status.set(Some("Record saved successfully!".to_string()));
                    // Navigate back to home after 1 second
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    nav.push("/");
                }
                Err(e) => {
                    save_status.set(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                Link { to: "/", class: "govuk-link", "← Back to home" }

                h1 { class: "govuk-heading-xl", "Add New Record" }

                if let Some(status) = save_status() {
                    div {
                        class: if status.contains("Error") { "result-card" } else { "result-card" },
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
                        h2 { class: "govuk-heading-l", "Personal Details" }

                        GovukInput {
                            label: "Full Name".to_string(),
                            hint: Some("As recorded in court documents".to_string()),
                            value: record().full_name.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.full_name = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-30".to_string()),
                        }

                        h2 { class: "govuk-heading-l", "Conviction Details" }

                        GovukInput {
                            label: "Offense Type".to_string(),
                            hint: Some("e.g., Assault, Theft, Fraud".to_string()),
                            value: record().offense_type.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.offense_type = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-30".to_string()),
                        }

                        GovukInput {
                            label: "Conviction Date".to_string(),
                            hint: Some("YYYY-MM-DD".to_string()),
                            value: record().conviction_date.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.conviction_date = val;
                                record.set(r);
                            },
                            input_type: Some("date".to_string()),
                            width_class: Some("govuk-input--width-10".to_string()),
                        }

                        GovukInput {
                            label: "Court Name".to_string(),
                            hint: Some("e.g., Manchester Crown Court".to_string()),
                            value: record().court_name.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.court_name = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-30".to_string()),
                        }

                        GovukInput {
                            label: "Sentence".to_string(),
                            hint: Some("e.g., 3 years imprisonment".to_string()),
                            value: record().sentence.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.sentence = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-30".to_string()),
                        }

                        h2 { class: "govuk-heading-l", "Location Details" }
                        p { class: "govuk-body govuk-hint",
                            "Only record street name and district as provided in public court records. Do not include house numbers."
                        }

                        GovukInput {
                            label: "Street Name".to_string(),
                            hint: Some("Street name only, no house number".to_string()),
                            value: record().street_name.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.street_name = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-30".to_string()),
                        }

                        GovukInput {
                            label: "City/Town".to_string(),
                            hint: None,
                            value: record().city.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.city = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-20".to_string()),
                        }

                        GovukInput {
                            label: "Postcode District".to_string(),
                            hint: Some("First part only (e.g., M1, SW1A)".to_string()),
                            value: record().postcode_district.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.postcode_district = val;
                                record.set(r);
                            },
                            input_type: None,
                            width_class: Some("govuk-input--width-10".to_string()),
                        }

                        h3 { class: "govuk-heading-m", "Map Coordinates (Optional)" }

                        div { class: "govuk-grid-row",
                            div { class: "govuk-grid-column-one-half",
                                GovukInput {
                                    label: "Latitude".to_string(),
                                    hint: Some("For map display".to_string()),
                                    value: record().latitude.map(|v| v.to_string()).unwrap_or_default(),
                                    oninput: move |val: String| {
                                        let mut r = record();
                                        r.latitude = val.parse::<f64>().ok();
                                        record.set(r);
                                    },
                                    input_type: Some("number".to_string()),
                                    width_class: Some("govuk-input--width-10".to_string()),
                                }
                            }
                            div { class: "govuk-grid-column-one-half",
                                GovukInput {
                                    label: "Longitude".to_string(),
                                    hint: Some("For map display".to_string()),
                                    value: record().longitude.map(|v| v.to_string()).unwrap_or_default(),
                                    oninput: move |val: String| {
                                        let mut r = record();
                                        r.longitude = val.parse::<f64>().ok();
                                        record.set(r);
                                    },
                                    input_type: Some("number".to_string()),
                                    width_class: Some("govuk-input--width-10".to_string()),
                                }
                            }
                        }

                        h2 { class: "govuk-heading-l", "Source & Consent" }

                        GovukSelect {
                            label: "Record Source".to_string(),
                            value: match record().source {
                                RecordSource::CourtRecord => "court",
                                RecordSource::Interview => "interview",
                                RecordSource::Both => "both",
                            }.to_string(),
                            options: vec![
                                ("court".to_string(), "Court Record".to_string()),
                                ("interview".to_string(), "Interview".to_string()),
                                ("both".to_string(), "Both".to_string()),
                            ],
                            onchange: move |val: String| {
                                let mut r = record();
                                r.source = match val.as_str() {
                                    "interview" => RecordSource::Interview,
                                    "both" => RecordSource::Both,
                                    _ => RecordSource::CourtRecord,
                                };
                                record.set(r);
                            },
                        }

                        div { class: "govuk-checkboxes",
                            GovukCheckbox {
                                label: "Interview consent obtained".to_string(),
                                checked: record().interview_consent,
                                onchange: move |val| {
                                    let mut r = record();
                                    r.interview_consent = val;
                                    record.set(r);
                                },
                            }
                        }

                        if record().interview_consent {
                            GovukInput {
                                label: "Interview Date".to_string(),
                                hint: Some("YYYY-MM-DD".to_string()),
                                value: record().interview_date.clone().unwrap_or_default(),
                                oninput: move |val: String| {
                                    let mut r = record();
                                    r.interview_date = if val.is_empty() { None } else { Some(val) };
                                    record.set(r);
                                },
                                input_type: Some("date".to_string()),
                                width_class: Some("govuk-input--width-10".to_string()),
                            }
                        }

                        GovukTextarea {
                            label: "Additional Notes".to_string(),
                            hint: Some("Any additional context or information".to_string()),
                            value: record().notes.clone(),
                            oninput: move |val| {
                                let mut r = record();
                                r.notes = val;
                                record.set(r);
                            },
                        }

                        GovukButton {
                            text: "Save Record".to_string(),
                            onclick: save_record,
                            button_type: None,
                        }

                        Link {
                            to: "/",
                            class: "govuk-button govuk-button--secondary",
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}

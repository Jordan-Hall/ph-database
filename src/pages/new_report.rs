use dioxus::prelude::*;
use crate::api::{ApiClient, CreateReportRequest};
use crate::notifications::NotificationService;

#[component]
pub fn NewReport() -> Element {
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut subject_name = use_signal(|| String::new());
    let mut offense_type = use_signal(|| String::new());
    let mut harm_risk = use_signal(|| String::from("low"));
    let mut loading = use_signal(|| false);
    let mut notifications = use_context::<NotificationService>();
    let nav = navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let request = CreateReportRequest {
            title: title(),
            description: description(),
            subject_name: subject_name(),
            offense_type: offense_type(),
            harm_risk: harm_risk(),
        };

        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.create_report(request).await {
                Ok(report) => {
                    notifications.success("Report created successfully!");
                    if let Some(id) = report.id {
                        nav.push(format!("/reports/{}", id));
                    } else {
                        nav.push("/reports");
                    }
                }
                Err(e) => {
                    notifications.error(format!("Failed to create report: {}", e));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Create new report" }

                form { onsubmit: on_submit,
                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "title", "Report title" }
                        input {
                            class: "govuk-input",
                            id: "title",
                            name: "title",
                            r#type: "text",
                            required: true,
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                            disabled: loading(),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "subject_name", "Subject name" }
                        input {
                            class: "govuk-input",
                            id: "subject_name",
                            name: "subject_name",
                            r#type: "text",
                            required: true,
                            value: "{subject_name}",
                            oninput: move |evt| subject_name.set(evt.value()),
                            disabled: loading(),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "offense_type", "Offense type" }
                        input {
                            class: "govuk-input",
                            id: "offense_type",
                            name: "offense_type",
                            r#type: "text",
                            required: true,
                            value: "{offense_type}",
                            oninput: move |evt| offense_type.set(evt.value()),
                            disabled: loading(),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "description", "Description" }
                        div { class: "govuk-hint", "Provide details about the incident" }
                        textarea {
                            class: "govuk-textarea",
                            id: "description",
                            name: "description",
                            rows: "10",
                            required: true,
                            value: "{description}",
                            oninput: move |evt| description.set(evt.value()),
                            disabled: loading(),
                        }
                    }

                    div { class: "govuk-form-group",
                        fieldset { class: "govuk-fieldset",
                            legend { class: "govuk-fieldset__legend govuk-fieldset__legend--m",
                                "Harm risk assessment"
                            }
                            div { class: "govuk-radios",
                                div { class: "govuk-radios__item",
                                    input {
                                        class: "govuk-radios__input",
                                        id: "harm-low",
                                        name: "harm_risk",
                                        r#type: "radio",
                                        value: "low",
                                        checked: harm_risk() == "low",
                                        onchange: move |_| harm_risk.set("low".to_string()),
                                        disabled: loading(),
                                    }
                                    label { class: "govuk-label govuk-radios__label", r#for: "harm-low", "Low" }
                                }
                                div { class: "govuk-radios__item",
                                    input {
                                        class: "govuk-radios__input",
                                        id: "harm-medium",
                                        name: "harm_risk",
                                        r#type: "radio",
                                        value: "medium",
                                        checked: harm_risk() == "medium",
                                        onchange: move |_| harm_risk.set("medium".to_string()),
                                        disabled: loading(),
                                    }
                                    label { class: "govuk-label govuk-radios__label", r#for: "harm-medium", "Medium" }
                                }
                                div { class: "govuk-radios__item",
                                    input {
                                        class: "govuk-radios__input",
                                        id: "harm-high",
                                        name: "harm_risk",
                                        r#type: "radio",
                                        value: "high",
                                        checked: harm_risk() == "high",
                                        onchange: move |_| harm_risk.set("high".to_string()),
                                        disabled: loading(),
                                    }
                                    label { class: "govuk-label govuk-radios__label", r#for: "harm-high", "High" }
                                }
                            }
                        }
                    }

                    button {
                        class: "govuk-button",
                        r#type: "submit",
                        disabled: loading(),
                        if loading() { "Creating report..." } else { "Create report" }
                    }

                    Link { to: "/reports", class: "govuk-link", style: "margin-left: 20px;", "Cancel" }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use crate::api::{ApiClient, CreateAlertRequest};
use crate::notifications::NotificationService;

#[component]
pub fn NewAlert() -> Element {
    let mut full_name = use_signal(|| String::new());
    let mut age = use_signal(|| String::new());
    let mut last_seen = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut contact = use_signal(|| String::new());
    let mut priority = use_signal(|| String::from("medium"));
    let mut loading = use_signal(|| false);
    let mut notifications = use_context::<NotificationService>();
    let nav = navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let age_val = age().parse::<i32>().ok();
        let request = CreateAlertRequest {
            full_name: full_name(),
            age: age_val,
            last_seen_location: last_seen(),
            description: description(),
            priority: priority(),
            contact_info: Some(contact()),
        };

        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.create_alert(request).await {
                Ok(_) => {
                    notifications.success("Alert created successfully!");
                    nav.push("/alerts/manage");
                }
                Err(e) => {
                    notifications.error(format!("Failed to create alert: {}", e));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Create Missing Person Alert" }

                form { onsubmit: on_submit,
                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "name", "Full name" }
                        input {
                            class: "govuk-input",
                            id: "name",
                            r#type: "text",
                            required: true,
                            value: "{full_name}",
                            oninput: move |evt| full_name.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "age", "Age (optional)" }
                        input {
                            class: "govuk-input govuk-input--width-5",
                            id: "age",
                            r#type: "number",
                            value: "{age}",
                            oninput: move |evt| age.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "last_seen", "Last seen location" }
                        input {
                            class: "govuk-input",
                            id: "last_seen",
                            r#type: "text",
                            required: true,
                            value: "{last_seen}",
                            oninput: move |evt| last_seen.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "description", "Description" }
                        textarea {
                            class: "govuk-textarea",
                            id: "description",
                            rows: "5",
                            required: true,
                            value: "{description}",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "contact", "Contact information" }
                        input {
                            class: "govuk-input",
                            id: "contact",
                            r#type: "text",
                            value: "{contact}",
                            oninput: move |evt| contact.set(evt.value()),
                        }
                    }

                    button {
                        class: "govuk-button",
                        r#type: "submit",
                        disabled: loading(),
                        if loading() { "Creating..." } else { "Create alert" }
                    }

                    Link { to: "/alerts/manage", class: "govuk-link", style: "margin-left: 20px;", "Cancel" }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use dioxus_router::Link;
use dioxus_router::hooks::use_navigator;
use crate::api::{ApiClient, SubmitStoryRequest};
use crate::notifications::NotificationService;

#[component]
pub fn SubmitStory() -> Element {
    let mut title = use_signal(|| String::new());
    let mut content = use_signal(|| String::new());
    let mut pseudonym = use_signal(|| String::new());
    let mut trigger_warning = use_signal(|| String::new());
    let mut consent = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut notifications = use_context::<NotificationService>();
    let nav = use_navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        if !consent() {
            notifications.error("You must give consent to submit your story");
            return;
        }

        let request = SubmitStoryRequest {
            title: title(),
            content: content(),
            author_pseudonym: if pseudonym().is_empty() { None } else { Some(pseudonym()) },
            consent_given: true,
            trigger_warning: if trigger_warning().is_empty() { None } else { Some(trigger_warning()) },
        };

        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.submit_story(request).await {
                Ok(_) => {
                    notifications.success("Story submitted for review. Thank you for sharing.");
                    nav.push("/stories");
                }
                Err(e) => {
                    notifications.error(format!("Failed to submit story: {}", e));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Submit Your Story" }

                div { class: "govuk-inset-text",
                    p { "Your story will be reviewed before being published. You can choose to remain anonymous." }
                }

                form { onsubmit: on_submit,
                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "title", "Story title" }
                        input {
                            class: "govuk-input",
                            id: "title",
                            r#type: "text",
                            required: true,
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "content", "Your story" }
                        textarea {
                            class: "govuk-textarea",
                            id: "content",
                            rows: "15",
                            required: true,
                            value: "{content}",
                            oninput: move |evt| content.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "pseudonym", "Pseudonym (optional)" }
                        div { class: "govuk-hint", "Leave blank to post anonymously" }
                        input {
                            class: "govuk-input",
                            id: "pseudonym",
                            r#type: "text",
                            value: "{pseudonym}",
                            oninput: move |evt| pseudonym.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        label { class: "govuk-label govuk-label--m", r#for: "warning", "Trigger warning (optional)" }
                        input {
                            class: "govuk-input",
                            id: "warning",
                            r#type: "text",
                            value: "{trigger_warning}",
                            oninput: move |evt| trigger_warning.set(evt.value()),
                        }
                    }

                    div { class: "govuk-form-group",
                        div { class: "govuk-checkboxes",
                            div { class: "govuk-checkboxes__item",
                                input {
                                    class: "govuk-checkboxes__input",
                                    id: "consent",
                                    r#type: "checkbox",
                                    checked: consent(),
                                    onchange: move |evt| consent.set(evt.checked()),
                                }
                                label { class: "govuk-label govuk-checkboxes__label", r#for: "consent",
                                    "I consent to my story being reviewed and potentially published"
                                }
                            }
                        }
                    }

                    button {
                        class: "govuk-button",
                        r#type: "submit",
                        disabled: loading(),
                        if loading() { "Submitting..." } else { "Submit story" }
                    }

                    Link { to: "/stories", class: "govuk-link", style: "margin-left: 20px;", "Cancel" }
                }
            }
        }
    }
}

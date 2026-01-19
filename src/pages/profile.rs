use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::notifications::NotificationService;

#[component]
pub fn Profile() -> Element {
    let auth = use_auth();
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut notifications = use_context::<NotificationService>();

    // Load current user data
    use_effect(move || {
        if let Some(user) = auth.user() {
            username.set(user.username.clone());
            email.set(user.email.clone());
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.update_profile(username(), email()).await {
                Ok(_) => {
                    notifications.success("Profile updated successfully!");
                }
                Err(e) => {
                    notifications.error(format!("Failed to update profile: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let user = auth.user();

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Your Profile" }

                if let Some(u) = user {
                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-two-thirds",
                            form { onsubmit: on_submit,
                                div { class: "govuk-form-group",
                                    label { class: "govuk-label govuk-label--m", r#for: "username", "Username" }
                                    input {
                                        class: "govuk-input",
                                        id: "username",
                                        r#type: "text",
                                        value: "{username}",
                                        oninput: move |evt| username.set(evt.value()),
                                        disabled: loading(),
                                    }
                                }

                                div { class: "govuk-form-group",
                                    label { class: "govuk-label govuk-label--m", r#for: "email", "Email" }
                                    input {
                                        class: "govuk-input",
                                        id: "email",
                                        r#type: "email",
                                        value: "{email}",
                                        oninput: move |evt| email.set(evt.value()),
                                        disabled: loading(),
                                    }
                                }

                                div { class: "govuk-form-group",
                                    label { class: "govuk-label govuk-label--m", "Roles" }
                                    ul { class: "govuk-list govuk-list--bullet",
                                        for role in &u.roles {
                                            li { "{role}" }
                                        }
                                    }
                                }

                                button {
                                    class: "govuk-button",
                                    r#type: "submit",
                                    disabled: loading(),
                                    if loading() { "Saving..." } else { "Save changes" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

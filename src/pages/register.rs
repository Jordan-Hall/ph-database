use dioxus::prelude::*;
use dioxus_router::Link;
use dioxus_router::hooks::use_navigator;
use crate::auth::use_auth;
use crate::notifications::NotificationService;

#[component]
pub fn Register() -> Element {
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut password_confirm = use_signal(|| String::new());
    let mut auth = use_auth();
    let mut notifications = use_context::<NotificationService>();
    let nav = use_navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let username_val = username();
        let email_val = email();
        let password_val = password();
        let password_confirm_val = password_confirm();

        // Validate passwords match
        if password_val != password_confirm_val {
            notifications.error("Passwords do not match");
            return;
        }

        // Validate password length
        if password_val.len() < 8 {
            notifications.error("Password must be at least 8 characters");
            return;
        }

        spawn(async move {
            match auth.register(username_val, email_val, password_val).await {
                Ok(_) => {
                    notifications.success("Registration successful! Welcome!");
                    nav.push("/dashboard");
                }
                Err(e) => {
                    notifications.error(format!("Registration failed: {}", e));
                }
            }
        });
    };

    let is_loading = auth.is_loading();

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                div { class: "govuk-grid-row",
                    div { class: "govuk-grid-column-two-thirds",
                        h1 { class: "govuk-heading-xl", "Create an account" }

                        p { class: "govuk-body",
                            "Join the Predator Hunters Database to report incidents, view alerts, and contribute to community safety."
                        }

                        form { onsubmit: on_submit,
                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "username",
                                    "Username"
                                }
                                div { class: "govuk-hint", id: "username-hint",
                                    "This will be publicly visible on your reports"
                                }
                                input {
                                    class: "govuk-input",
                                    id: "username",
                                    name: "username",
                                    r#type: "text",
                                    required: true,
                                    "aria-describedby": "username-hint",
                                    value: "{username}",
                                    oninput: move |evt| username.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "email",
                                    "Email address"
                                }
                                div { class: "govuk-hint", id: "email-hint",
                                    "We'll use this to send you alerts and notifications"
                                }
                                input {
                                    class: "govuk-input govuk-input--width-20",
                                    id: "email",
                                    name: "email",
                                    r#type: "email",
                                    required: true,
                                    autocomplete: "email",
                                    "aria-describedby": "email-hint",
                                    value: "{email}",
                                    oninput: move |evt| email.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "password",
                                    "Password"
                                }
                                div { class: "govuk-hint", id: "password-hint",
                                    "Must be at least 8 characters"
                                }
                                input {
                                    class: "govuk-input govuk-input--width-20",
                                    id: "password",
                                    name: "password",
                                    r#type: "password",
                                    required: true,
                                    minlength: 8,
                                    "aria-describedby": "password-hint",
                                    value: "{password}",
                                    oninput: move |evt| password.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "password-confirm",
                                    "Confirm password"
                                }
                                input {
                                    class: "govuk-input govuk-input--width-20",
                                    id: "password-confirm",
                                    name: "password-confirm",
                                    r#type: "password",
                                    required: true,
                                    minlength: 8,
                                    value: "{password_confirm}",
                                    oninput: move |evt| password_confirm.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            div { class: "govuk-warning-text",
                                span { class: "govuk-warning-text__icon", "aria-hidden": "true", "!" }
                                strong { class: "govuk-warning-text__text",
                                    span { class: "govuk-warning-text__assistive", "Warning" }
                                    "By creating an account, you agree to use this platform responsibly and in accordance with UK law."
                                }
                            }

                            button {
                                class: "govuk-button",
                                r#type: "submit",
                                "data-module": "govuk-button",
                                disabled: is_loading,
                                if is_loading {
                                    "Creating account..."
                                } else {
                                    "Create account"
                                }
                            }
                        }

                        hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                        p { class: "govuk-body",
                            "Already have an account? "
                            Link { to: "/login", "Sign in" }
                        }
                    }
                }
            }
        }
    }
}

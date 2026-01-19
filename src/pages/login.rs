use dioxus::prelude::*;
use crate::auth::use_auth;
use crate::notifications::NotificationService;

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut auth = use_auth();
    let mut notifications = use_context::<NotificationService>();
    let nav = navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let email_val = email();
        let password_val = password();

        spawn(async move {
            match auth.login(email_val, password_val).await {
                Ok(_) => {
                    notifications.success("Login successful!");
                    nav.push("/dashboard");
                }
                Err(e) => {
                    notifications.error(format!("Login failed: {}", e));
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
                        h1 { class: "govuk-heading-xl", "Sign in" }

                        form { onsubmit: on_submit,
                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "email", "Email address" }
                                input {
                                    class: "govuk-input",
                                    id: "email",
                                    name: "email",
                                    r#type: "email",
                                    required: true,
                                    value: "{email}",
                                    oninput: move |evt| email.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            div { class: "govuk-form-group",
                                label { class: "govuk-label", r#for: "password", "Password" }
                                input {
                                    class: "govuk-input",
                                    id: "password",
                                    name: "password",
                                    r#type: "password",
                                    required: true,
                                    value: "{password}",
                                    oninput: move |evt| password.set(evt.value()),
                                    disabled: is_loading,
                                }
                            }

                            button {
                                class: "govuk-button",
                                r#type: "submit",
                                disabled: is_loading,
                                if is_loading {
                                    "Signing in..."
                                } else {
                                    "Sign in"
                                }
                            }
                        }

                        hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                        p { class: "govuk-body",
                            "Don't have an account? "
                            Link { to: "/register", "Create an account" }
                        }
                    }
                }
            }
        }
    }
}

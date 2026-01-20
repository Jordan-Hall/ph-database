use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::notifications::NotificationService;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MfaSetupResponse {
    secret: String,
    qr_code_url: String,
    backup_codes: Vec<String>,
    manual_entry_key: String,
}

#[component]
pub fn MfaSettings() -> Element {
    let auth = use_auth();
    let mut loading = use_signal(|| false);
    let mut mfa_enabled = use_signal(|| false);
    let mut setup_mode = use_signal(|| false);
    let mut mfa_setup_data = use_signal(|| None::<MfaSetupResponse>);
    let mut verification_code = use_signal(|| String::new());
    let mut backup_codes = use_signal(|| None::<Vec<String>>);
    let mut notifications = use_context::<NotificationService>();

    // Check if user has MFA enabled
    use_effect(move || {
        if let Some(user) = auth.user() {
            // Assuming the user model has an mfa_enabled field
            // You may need to fetch this from the API
            mfa_enabled.set(false); // TODO: Get from user object or API
        }
    });

    let start_mfa_setup = move |_| {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            match api.post::<(), MfaSetupResponse>("/api/v1/auth/mfa/setup", &()).await {
                Ok(response) => {
                    mfa_setup_data.set(Some(response));
                    setup_mode.set(true);
                    notifications.success("MFA setup initiated. Scan the QR code with your authenticator app.");
                }
                Err(e) => {
                    notifications.error(format!("Failed to start MFA setup: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let enable_mfa = move |evt: Event<FormData>| {
        evt.prevent_default();

        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            #[derive(Serialize)]
            struct EnableRequest {
                code: String,
            }

            match api.post::<EnableRequest, serde_json::Value>(
                "/api/v1/auth/mfa/enable",
                &EnableRequest { code: verification_code() }
            ).await {
                Ok(_) => {
                    mfa_enabled.set(true);
                    setup_mode.set(false);
                    notifications.success("MFA enabled successfully!");

                    // Save backup codes for display
                    if let Some(setup_data) = mfa_setup_data() {
                        backup_codes.set(Some(setup_data.backup_codes));
                    }
                }
                Err(e) => {
                    notifications.error(format!("Failed to enable MFA: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let disable_mfa = move |_| {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            #[derive(Serialize)]
            struct DisableRequest {
                code: String,
            }

            // Prompt user for MFA code (simplified - in production use a modal)
            let code = "123456"; // TODO: Get from user input

            match api.post::<DisableRequest, serde_json::Value>(
                "/api/v1/auth/mfa/disable",
                &DisableRequest { code: code.to_string() }
            ).await {
                Ok(_) => {
                    mfa_enabled.set(false);
                    notifications.success("MFA disabled successfully!");
                }
                Err(e) => {
                    notifications.error(format!("Failed to disable MFA: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let regenerate_backup_codes = move |_| {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            #[derive(Deserialize)]
            struct BackupCodesResponse {
                backup_codes: Vec<String>,
            }

            match api.get::<BackupCodesResponse>("/api/v1/auth/mfa/backup-codes").await {
                Ok(response) => {
                    backup_codes.set(Some(response.backup_codes));
                    notifications.success("Backup codes regenerated successfully!");
                }
                Err(e) => {
                    notifications.error(format!("Failed to regenerate backup codes: {}", e));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Multi-Factor Authentication" }

                div { class: "govuk-grid-row",
                    div { class: "govuk-grid-column-two-thirds",

                        // MFA Status
                        div { class: "govuk-panel govuk-panel--confirmation",
                            h2 { class: "govuk-panel__title",
                                if mfa_enabled() {
                                    "MFA is enabled"
                                } else {
                                    "MFA is not enabled"
                                }
                            }
                            div { class: "govuk-panel__body",
                                if mfa_enabled() {
                                    "Your account is protected with multi-factor authentication"
                                } else {
                                    "Enable MFA to add an extra layer of security"
                                }
                            }
                        }

                        // Setup mode
                        if setup_mode() && !mfa_enabled() {
                            if let Some(setup_data) = mfa_setup_data() {
                                div { class: "govuk-!-margin-top-6",
                                    h2 { class: "govuk-heading-l", "Step 1: Scan QR Code" }

                                    p { class: "govuk-body",
                                        "Scan this QR code with your authenticator app (Google Authenticator, Authy, etc.)"
                                    }

                                    div { class: "govuk-!-margin-top-4 govuk-!-margin-bottom-4",
                                        img {
                                            src: "{setup_data.qr_code_url}",
                                            alt: "MFA QR Code",
                                            style: "max-width: 300px;"
                                        }
                                    }

                                    div { class: "govuk-inset-text",
                                        p { class: "govuk-body", strong { "Manual entry key:" } }
                                        code { class: "govuk-!-font-size-16", "{setup_data.manual_entry_key}" }
                                    }

                                    h2 { class: "govuk-heading-l govuk-!-margin-top-6", "Step 2: Verify Code" }

                                    form { onsubmit: enable_mfa,
                                        div { class: "govuk-form-group",
                                            label { class: "govuk-label", r#for: "verification-code",
                                                "Enter the 6-digit code from your authenticator app"
                                            }
                                            input {
                                                class: "govuk-input govuk-input--width-10",
                                                id: "verification-code",
                                                r#type: "text",
                                                value: "{verification_code}",
                                                oninput: move |evt| verification_code.set(evt.value()),
                                                disabled: loading(),
                                                placeholder: "000000",
                                                maxlength: "6",
                                            }
                                        }

                                        button {
                                            class: "govuk-button",
                                            r#type: "submit",
                                            disabled: loading() || verification_code().len() != 6,
                                            if loading() { "Verifying..." } else { "Enable MFA" }
                                        }
                                    }

                                    div { class: "govuk-warning-text govuk-!-margin-top-6",
                                        span { class: "govuk-warning-text__icon", "!", aria_hidden: "true" }
                                        strong { class: "govuk-warning-text__text",
                                            span { class: "govuk-warning-text__assistive", "Warning" }
                                            "Save your backup codes"
                                        }
                                    }

                                    div { class: "govuk-inset-text",
                                        p { class: "govuk-body", strong { "Backup codes (save these somewhere safe):" } }
                                        ul { class: "govuk-list",
                                            for code in &setup_data.backup_codes {
                                                li {
                                                    code { class: "govuk-!-font-size-16", "{code}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // MFA enabled state
                        if mfa_enabled() && !setup_mode() {
                            div { class: "govuk-!-margin-top-6",
                                h2 { class: "govuk-heading-m", "MFA Settings" }

                                p { class: "govuk-body",
                                    "Multi-factor authentication is currently enabled for your account. \
                                    You will be required to enter a code from your authenticator app when signing in."
                                }

                                div { class: "govuk-button-group govuk-!-margin-top-4",
                                    button {
                                        class: "govuk-button govuk-button--secondary",
                                        r#type: "button",
                                        onclick: regenerate_backup_codes,
                                        disabled: loading(),
                                        "Regenerate backup codes"
                                    }

                                    button {
                                        class: "govuk-button govuk-button--warning",
                                        r#type: "button",
                                        onclick: disable_mfa,
                                        disabled: loading(),
                                        "Disable MFA"
                                    }
                                }

                                // Show backup codes if regenerated
                                if let Some(codes) = backup_codes() {
                                    div { class: "govuk-inset-text govuk-!-margin-top-6",
                                        p { class: "govuk-body", strong { "New backup codes (save these somewhere safe):" } }
                                        ul { class: "govuk-list",
                                            for code in codes {
                                                li {
                                                    code { class: "govuk-!-font-size-16", "{code}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // MFA not enabled state
                        if !mfa_enabled() && !setup_mode() {
                            div { class: "govuk-!-margin-top-6",
                                h2 { class: "govuk-heading-m", "Why enable MFA?" }

                                p { class: "govuk-body",
                                    "Multi-factor authentication (MFA) adds an extra layer of security to your account. \
                                    Even if someone knows your password, they won't be able to access your account without \
                                    also having access to your authenticator app."
                                }

                                details { class: "govuk-details",
                                    summary { class: "govuk-details__summary",
                                        span { class: "govuk-details__summary-text", "How does it work?" }
                                    }
                                    div { class: "govuk-details__text",
                                        p { "When you enable MFA:" }
                                        ol { class: "govuk-list govuk-list--number",
                                            li { "You'll scan a QR code with an authenticator app on your phone" }
                                            li { "Each time you log in, you'll enter a 6-digit code from the app" }
                                            li { "You'll receive backup codes to use if you lose access to your phone" }
                                        }
                                    }
                                }

                                button {
                                    class: "govuk-button govuk-!-margin-top-4",
                                    r#type: "button",
                                    onclick: start_mfa_setup,
                                    disabled: loading(),
                                    if loading() { "Setting up..." } else { "Enable MFA" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

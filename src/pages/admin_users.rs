use dioxus::prelude::*;
use dioxus_router::Link;
use crate::api::{ApiClient, UserInfo};

#[component]
pub fn AdminUsers() -> Element {
    let mut users = use_signal(|| Vec::<UserInfo>::new());
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            if let Ok(data) = api.get_users().await {
                users.set(data);
            }
            
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Manage Users" }

                Link { to: "/admin", class: "govuk-back-link", "Back to admin dashboard" }

                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                if loading() {
                    p { class: "govuk-body", "Loading users..." }
                } else if users().is_empty() {
                    div { class: "govuk-inset-text",
                        p { "No users found." }
                    }
                } else {
                    table { class: "govuk-table",
                        thead { class: "govuk-table__head",
                            tr { class: "govuk-table__row",
                                th { class: "govuk-table__header", scope: "col", "Username" }
                                th { class: "govuk-table__header", scope: "col", "Email" }
                                th { class: "govuk-table__header", scope: "col", "Roles" }
                            }
                        }
                        tbody { class: "govuk-table__body",
                            for user in users() {
                                tr { class: "govuk-table__row",
                                    td { class: "govuk-table__cell", "{user.username}" }
                                    td { class: "govuk-table__cell", "{user.email}" }
                                    td { class: "govuk-table__cell", "{user.roles.join(\", \")}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use dioxus_router::Link;
use crate::api::ApiClient;

#[component]
pub fn AdminDashboard() -> Element {
    let mut users_count = use_signal(|| 0);
    let mut tenants_count = use_signal(|| 0);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            if let Ok(users) = api.get_users().await {
                users_count.set(users.len());
            }

            if let Ok(tenants) = api.get_tenants().await {
                tenants_count.set(tenants.len());
            }

            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Admin Dashboard" }

                if loading() {
                    p { class: "govuk-body", "Loading..." }
                } else {
                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-one-half",
                            div {
                                class: "govuk-panel govuk-panel--confirmation",
                                style: "background-color: #1d70b8; padding: 20px; margin-bottom: 20px;",
                                h2 { class: "govuk-panel__title", style: "font-size: 24px;", "Total Users" }
                                div { class: "govuk-panel__body", style: "font-size: 48px;", "{users_count()}" }
                            }
                        }

                        div { class: "govuk-grid-column-one-half",
                            div {
                                class: "govuk-panel",
                                style: "background-color: #00703c; color: white; padding: 20px; margin-bottom: 20px;",
                                h2 { class: "govuk-panel__title", style: "font-size: 24px; color: white;", "Business Tenants" }
                                div { class: "govuk-panel__body", style: "font-size: 48px; color: white;", "{tenants_count()}" }
                            }
                        }
                    }

                    hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                    h2 { class: "govuk-heading-l", "Admin actions" }

                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-one-third",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "User Management" }
                                }
                                div { class: "govuk-summary-card__content",
                                    Link { to: "/admin/users", class: "govuk-link", "Manage users" }
                                }
                            }
                        }

                        div { class: "govuk-grid-column-one-third",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Business Tenants" }
                                }
                                div { class: "govuk-summary-card__content",
                                    Link { to: "/admin/tenants", class: "govuk-link", "Manage tenants" }
                                }
                            }
                        }

                        div { class: "govuk-grid-column-one-third",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Review Queue" }
                                }
                                div { class: "govuk-summary-card__content",
                                    Link { to: "/admin/review", class: "govuk-link", "View review queue" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

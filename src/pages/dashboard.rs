use dioxus::prelude::*;
use crate::api::ApiClient;
use crate::auth::use_auth;
use crate::notifications::NotificationService;

#[component]
pub fn Dashboard() -> Element {
    let auth = use_auth();
    let mut reports_count = use_signal(|| 0);
    let mut alerts_count = use_signal(|| 0);
    let mut loading = use_signal(|| true);
    let mut notifications = use_context::<NotificationService>();

    // Load dashboard stats
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();

            // Load reports count
            if let Ok(reports) = api.get_reports().await {
                reports_count.set(reports.len());
            }

            // Load alerts count
            if let Ok(alerts) = api.get_active_alerts().await {
                alerts_count.set(alerts.len());
            }

            loading.set(false);
        });
    });

    let user = auth.user();
    let username = user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "User".to_string());
    let is_admin = auth.is_admin();
    let is_reviewer = auth.is_reviewer();

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Dashboard" }

                p { class: "govuk-body-l", "Welcome back, {username}!" }

                if loading() {
                    div { class: "govuk-body",
                        p { "Loading your dashboard..." }
                    }
                } else {
                    // Stats summary
                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-one-third",
                            div {
                                class: "govuk-panel govuk-panel--confirmation",
                                style: "background-color: #1d70b8; padding: 20px; margin-bottom: 20px;",
                                h2 { class: "govuk-panel__title", style: "font-size: 24px;", "Reports" }
                                div { class: "govuk-panel__body", style: "font-size: 48px;", "{reports_count()}" }
                            }
                        }

                        div { class: "govuk-grid-column-one-third",
                            div {
                                class: "govuk-panel",
                                style: "background-color: #d4351c; color: white; padding: 20px; margin-bottom: 20px;",
                                h2 { class: "govuk-panel__title", style: "font-size: 24px; color: white;", "Active Alerts" }
                                div { class: "govuk-panel__body", style: "font-size: 48px; color: white;", "{alerts_count()}" }
                            }
                        }

                        if is_reviewer || is_admin {
                            div { class: "govuk-grid-column-one-third",
                                div {
                                    class: "govuk-panel",
                                    style: "background-color: #00703c; color: white; padding: 20px; margin-bottom: 20px;",
                                    h2 { class: "govuk-panel__title", style: "font-size: 24px; color: white;", "Your Role" }
                                    div { class: "govuk-panel__body", style: "font-size: 24px; color: white;",
                                        if is_admin { "Administrator" }
                                        else if is_reviewer { "Reviewer" }
                                        else { "User" }
                                    }
                                }
                            }
                        }
                    }

                    hr { class: "govuk-section-break govuk-section-break--l govuk-section-break--visible" }

                    // Quick actions
                    h2 { class: "govuk-heading-l", "Quick actions" }

                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-one-half",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Reports" }
                                }
                                div { class: "govuk-summary-card__content",
                                    ul { class: "govuk-list govuk-list--bullet",
                                        li {
                                            Link { to: "/reports/new", class: "govuk-link", "Create new report" }
                                        }
                                        li {
                                            Link { to: "/reports", class: "govuk-link", "View all reports" }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "govuk-grid-column-one-half",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Alerts" }
                                }
                                div { class: "govuk-summary-card__content",
                                    ul { class: "govuk-list govuk-list--bullet",
                                        li {
                                            Link { to: "/alerts/new", class: "govuk-link", "Create missing person alert" }
                                        }
                                        li {
                                            Link { to: "/alerts/manage", class: "govuk-link", "Manage alerts" }
                                        }
                                        li {
                                            Link { to: "/alerts", class: "govuk-link", "View active public alerts" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-one-half",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Stories" }
                                }
                                div { class: "govuk-summary-card__content",
                                    ul { class: "govuk-list govuk-list--bullet",
                                        li {
                                            Link { to: "/stories/submit", class: "govuk-link", "Submit your story" }
                                        }
                                        li {
                                            Link { to: "/stories", class: "govuk-link", "Read survivor stories" }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "govuk-grid-column-one-half",
                            div { class: "govuk-summary-card",
                                div { class: "govuk-summary-card__title-wrapper",
                                    h3 { class: "govuk-summary-card__title", "Other" }
                                }
                                div { class: "govuk-summary-card__content",
                                    ul { class: "govuk-list govuk-list--bullet",
                                        li {
                                            Link { to: "/map", class: "govuk-link", "View map" }
                                        }
                                        li {
                                            Link { to: "/profile", class: "govuk-link", "Edit profile" }
                                        }
                                        if is_admin {
                                            li {
                                                Link { to: "/admin", class: "govuk-link", "Admin dashboard" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

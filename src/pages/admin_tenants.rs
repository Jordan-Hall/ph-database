use dioxus::prelude::*;
use crate::api::{ApiClient, BusinessTenant};

#[component]
pub fn AdminTenants() -> Element {
    let mut tenants = use_signal(|| Vec::<BusinessTenant>::new());
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            if let Ok(data) = api.get_tenants().await {
                tenants.set(data);
            }
            
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Manage Business Tenants" }

                Link { to: "/admin", class: "govuk-back-link", "Back to admin dashboard" }

                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                if loading() {
                    p { class: "govuk-body", "Loading tenants..." }
                } else if tenants().is_empty() {
                    div { class: "govuk-inset-text",
                        p { "No business tenants found." }
                    }
                } else {
                    for tenant in tenants() {
                        div { class: "govuk-summary-card",
                            div { class: "govuk-summary-card__title-wrapper",
                                h2 { class: "govuk-summary-card__title", "{tenant.company_name}" }
                            }
                            div { class: "govuk-summary-card__content",
                                dl { class: "govuk-summary-list",
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Contact Email" }
                                        dd { class: "govuk-summary-list__value", "{tenant.contact_email}" }
                                    }
                                    div { class: "govuk-summary-list__row",
                                        dt { class: "govuk-summary-list__key", "Status" }
                                        dd { class: "govuk-summary-list__value",
                                            strong { class: "govuk-tag", "{tenant.status}" }
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

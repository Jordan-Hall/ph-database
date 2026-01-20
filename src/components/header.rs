use dioxus::prelude::*;
use dioxus_router::Link;
use crate::auth::use_auth;

#[component]
pub fn Header() -> Element {
    let auth = use_auth();
    let is_authenticated = auth.is_authenticated();
    let is_admin = auth.is_admin();
    let user = auth.user();

    rsx! {
        header { class: "govuk-header", role: "banner", "data-module": "govuk-header",
            div { class: "govuk-header__container govuk-width-container",
                div { class: "govuk-header__logo",
                    Link {
                        to: "/",
                        class: "govuk-header__link govuk-header__link--homepage",
                        span { class: "govuk-header__logotype",
                            span { class: "govuk-header__logotype-text", "GOV.UK" }
                        }
                    }
                }
                div { class: "govuk-header__content",
                    Link {
                        to: "/",
                        class: "govuk-header__link govuk-header__link--service-name",
                        "Predator Hunters Database"
                    }

                    button {
                        r#type: "button",
                        class: "govuk-header__menu-button govuk-js-header-toggle",
                        "aria-controls": "navigation",
                        "aria-label": "Show or hide navigation menu",
                        hidden: true,
                        "Menu"
                    }

                    nav { "aria-label": "Navigation",
                        ul { class: "govuk-header__navigation", id: "navigation",
                            if is_authenticated {
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/dashboard",
                                        class: "govuk-header__link",
                                        "Dashboard"
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/reports",
                                        class: "govuk-header__link",
                                        "Reports"
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/alerts",
                                        class: "govuk-header__link",
                                        "Alerts"
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/stories",
                                        class: "govuk-header__link",
                                        "Stories"
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/map",
                                        class: "govuk-header__link",
                                        "Map"
                                    }
                                }
                                if is_admin {
                                    li { class: "govuk-header__navigation-item",
                                        Link {
                                            to: "/admin",
                                            class: "govuk-header__link",
                                            "Admin"
                                        }
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/profile",
                                        class: "govuk-header__link",
                                        if let Some(u) = user.as_ref() {
                                            "{u.username}"
                                        } else {
                                            "Profile"
                                        }
                                    }
                                }
                            } else {
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/login",
                                        class: "govuk-header__link",
                                        "Sign in"
                                    }
                                }
                                li { class: "govuk-header__navigation-item",
                                    Link {
                                        to: "/register",
                                        class: "govuk-header__link",
                                        "Register"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        div { class: "govuk-width-container",
            div { class: "govuk-phase-banner",
                p { class: "govuk-phase-banner__content",
                    strong { class: "govuk-tag govuk-phase-banner__content__tag",
                        "beta"
                    }
                    span { class: "govuk-phase-banner__text",
                        "This is a new service – your "
                        a { class: "govuk-link", href: "mailto:feedback@example.gov.uk", "feedback" }
                        " will help us to improve it."
                    }
                }
            }
        }
    }
}

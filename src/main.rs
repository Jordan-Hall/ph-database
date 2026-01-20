mod api;
mod auth;
mod components;
mod database;
mod models;
mod notifications;
mod pages;

use dioxus::prelude::*;
use dioxus_router::{Routable, Router, Link};

use auth::{AuthState, RequireAdmin, RequireAuth};
use components::{Header, Footer};
use notifications::{NotificationContainer, NotificationService};

fn main() {
    // Initialize logging
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    dioxus::launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    // Public routes
    #[route("/")]
    HomePage {},

    #[route("/login")]
    LoginPage {},

    #[route("/register")]
    RegisterPage {},

    #[route("/items/:slug")]
    PublishedItemPage { slug: String },

    #[route("/stories")]
    StoriesPage {},

    #[route("/alerts")]
    PublicAlertsPage {},

    // Protected routes
    #[route("/dashboard")]
    DashboardPage {},

    #[route("/reports")]
    ReportsPage {},

    #[route("/reports/new")]
    NewReportPage {},

    #[route("/reports/:id")]
    ReportDetailPage { id: String },

    #[route("/alerts/manage")]
    ManageAlertsPage {},

    #[route("/alerts/new")]
    NewAlertPage {},

    #[route("/map")]
    MapPage {},

    #[route("/stories/submit")]
    SubmitStoryPage {},

    #[route("/profile")]
    ProfilePage {},

    // Admin routes
    #[route("/admin")]
    AdminDashboardPage {},

    #[route("/admin/users")]
    AdminUsersPage {},

    #[route("/admin/tenants")]
    AdminTenantsPage {},

    #[route("/admin/review")]
    AdminReviewPage {},

    // Legacy routes (keeping for compatibility)
    #[route("/add")]
    AddRecordPage {},

    #[route("/view/:id")]
    ViewRecordPage { id: String },
}

#[component]
fn App() -> Element {
    // Initialize global state
    use_context_provider(|| AuthState::new());
    use_context_provider(|| NotificationService::new());

    // Initialize auth on app load
    let mut auth = use_context::<AuthState>();
    use_effect(move || {
        spawn(async move {
            auth.init().await;
        });
    });

    rsx! {
        Router::<Route> {}
        NotificationContainer {}
    }
}

// ============================================================================
// Layout Component
// ============================================================================

#[component]
fn Layout(children: Element) -> Element {
    rsx! {
        Header {}
        {children}
        Footer {}
    }
}

// ============================================================================
// Public Routes
// ============================================================================

#[component]
fn HomePage() -> Element {
    rsx! {
        Layout {
            pages::Home {}
        }
    }
}

#[component]
fn LoginPage() -> Element {
    rsx! {
        Layout {
            pages::Login {}
        }
    }
}

#[component]
fn RegisterPage() -> Element {
    rsx! {
        Layout {
            pages::Register {}
        }
    }
}

#[component]
fn PublishedItemPage(slug: String) -> Element {
    rsx! {
        Layout {
            pages::PublishedItem { slug }
        }
    }
}

#[component]
fn StoriesPage() -> Element {
    rsx! {
        Layout {
            pages::Stories {}
        }
    }
}

#[component]
fn PublicAlertsPage() -> Element {
    rsx! {
        Layout {
            pages::PublicAlerts {}
        }
    }
}

// ============================================================================
// Protected Routes
// ============================================================================

#[component]
fn DashboardPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::Dashboard {}
            }
        }
    }
}

#[component]
fn ReportsPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::Reports {}
            }
        }
    }
}

#[component]
fn NewReportPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::NewReport {}
            }
        }
    }
}

#[component]
fn ReportDetailPage(id: String) -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::ReportDetail { id }
            }
        }
    }
}

#[component]
fn ManageAlertsPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::ManageAlerts {}
            }
        }
    }
}

#[component]
fn NewAlertPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::NewAlert {}
            }
        }
    }
}

#[component]
fn MapPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::MapView {}
            }
        }
    }
}

#[component]
fn SubmitStoryPage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::SubmitStory {}
            }
        }
    }
}

#[component]
fn ProfilePage() -> Element {
    rsx! {
        RequireAuth {
            Layout {
                pages::Profile {}
            }
        }
    }
}

// ============================================================================
// Admin Routes
// ============================================================================

#[component]
fn AdminDashboardPage() -> Element {
    rsx! {
        RequireAdmin {
            Layout {
                pages::AdminDashboard {}
            }
        }
    }
}

#[component]
fn AdminUsersPage() -> Element {
    rsx! {
        RequireAdmin {
            Layout {
                pages::AdminUsers {}
            }
        }
    }
}

#[component]
fn AdminTenantsPage() -> Element {
    rsx! {
        RequireAdmin {
            Layout {
                pages::AdminTenants {}
            }
        }
    }
}

#[component]
fn AdminReviewPage() -> Element {
    rsx! {
        RequireAdmin {
            Layout {
                pages::AdminReview {}
            }
        }
    }
}

// ============================================================================
// Legacy Routes
// ============================================================================

#[component]
fn AddRecordPage() -> Element {
    rsx! {
        Layout {
            pages::AddRecord {}
        }
    }
}

#[component]
fn ViewRecordPage(id: String) -> Element {
    rsx! {
        Layout {
            pages::ViewRecord { id }
        }
    }
}

use crate::api::{ApiClient, UserInfo};
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

#[derive(Clone, Copy)]
pub struct AuthState {
    user: Signal<Option<UserInfo>>,
    loading: Signal<bool>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            user: Signal::new(None),
            loading: Signal::new(false),
        }
    }

    /// Get the current user
    pub fn user(&self) -> Option<UserInfo> {
        self.user.read().clone()
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user.read().is_some()
    }

    /// Check if user is loading
    pub fn is_loading(&self) -> bool {
        *self.loading.read()
    }

    /// Set the current user
    pub fn set_user(&mut self, user: Option<UserInfo>) {
        *self.user.write() = user;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        *self.loading.write() = loading;
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        if let Some(user) = self.user.read().as_ref() {
            user.roles.contains(&role.to_string())
        } else {
            false
        }
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    /// Check if user is reviewer
    pub fn is_reviewer(&self) -> bool {
        self.has_role("reviewer") || self.is_admin()
    }

    /// Check if user is publisher
    pub fn is_publisher(&self) -> bool {
        self.has_role("publisher") || self.is_admin()
    }

    /// Initialize authentication state from stored token
    pub async fn init(&mut self) {
        let api = ApiClient::new();

        // Check if we have a token
        if !api.is_authenticated() {
            return;
        }

        self.set_loading(true);

        // Try to fetch current user info
        match api.get_current_user().await {
            Ok(user) => {
                log::info!("User authenticated: {}", user.username);
                self.set_user(Some(user));
            }
            Err(e) => {
                log::error!("Failed to fetch user: {}", e);
                // Token might be expired, clear it
                api.clear_token();
                self.set_user(None);
            }
        }

        self.set_loading(false);
    }

    /// Login with email and password
    pub async fn login(&mut self, email: String, password: String) -> Result<(), String> {
        self.set_loading(true);

        let api = ApiClient::new();
        let result = api.login(email, password).await;

        match result {
            Ok(response) => {
                log::info!("Login successful: {}", response.user.username);
                self.set_user(Some(response.user));
                self.set_loading(false);
                Ok(())
            }
            Err(e) => {
                log::error!("Login failed: {}", e);
                self.set_loading(false);
                Err(e)
            }
        }
    }

    /// Register a new user
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(), String> {
        self.set_loading(true);

        let api = ApiClient::new();
        let result = api.register(username, email, password).await;

        match result {
            Ok(response) => {
                log::info!("Registration successful: {}", response.user.username);
                self.set_user(Some(response.user));
                self.set_loading(false);
                Ok(())
            }
            Err(e) => {
                log::error!("Registration failed: {}", e);
                self.set_loading(false);
                Err(e)
            }
        }
    }

    /// Logout the current user
    pub async fn logout(&mut self) {
        let api = ApiClient::new();

        if let Err(e) = api.logout().await {
            log::error!("Logout failed: {}", e);
        }

        self.set_user(None);
        log::info!("User logged out");
    }

    /// Refresh the authentication token
    pub async fn refresh_token(&mut self) -> Result<(), String> {
        let api = ApiClient::new();

        match api.refresh_token().await {
            Ok(response) => {
                self.set_user(Some(response.user));
                Ok(())
            }
            Err(e) => {
                // Token refresh failed, clear auth
                api.clear_token();
                self.set_user(None);
                Err(e)
            }
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook to use authentication state
pub fn use_auth() -> AuthState {
    use_context::<AuthState>()
}

/// Component to require authentication
#[component]
pub fn RequireAuth(children: Element) -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    if !auth.is_authenticated() {
        // Redirect to login
        nav.push("/login");
        return rsx! {
            div { class: "govuk-width-container",
                div { class: "govuk-main-wrapper",
                    p { class: "govuk-body", "Redirecting to login..." }
                }
            }
        };
    }

    rsx! { {children} }
}

/// Component to require admin role
#[component]
pub fn RequireAdmin(children: Element) -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    if !auth.is_authenticated() {
        nav.push("/login");
        return rsx! {
            div { class: "govuk-width-container",
                div { class: "govuk-main-wrapper",
                    p { class: "govuk-body", "Redirecting to login..." }
                }
            }
        };
    }

    if !auth.is_admin() {
        return rsx! {
            div { class: "govuk-width-container",
                div { class: "govuk-main-wrapper",
                    h1 { class: "govuk-heading-xl", "Access Denied" }
                    p { class: "govuk-body", "You don't have permission to access this page." }
                    a { class: "govuk-link", href: "/", "Return to home" }
                }
            }
        };
    }

    rsx! { {children} }
}

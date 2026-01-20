use super::types::*;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::Serialize;
use web_sys::Headers;

const API_BASE_URL: &str = "http://localhost:8080";
const TOKEN_KEY: &str = "auth_token";

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: API_BASE_URL.to_string(),
        }
    }

    /// Get the stored authentication token
    pub fn get_token(&self) -> Option<String> {
        LocalStorage::get(TOKEN_KEY).ok()
    }

    /// Set the authentication token
    pub fn set_token(&self, token: &str) {
        LocalStorage::set(TOKEN_KEY, token).ok();
    }

    /// Clear the authentication token
    pub fn clear_token(&self) {
        LocalStorage::delete(TOKEN_KEY);
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.get_token().is_some()
    }

    /// Generic GET request
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = Request::get(&url);

        // TODO: Add authentication header support for gloo-net 0.6
        // The header API changed in gloo-net 0.6 - needs investigation
        if let Some(_token) = self.get_token() {
            // Authentication headers temporarily disabled
            // Need to use web_sys Headers API or different gloo-net method
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let error: ApiError = response
                .json()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown".to_string(),
                    message: format!("Request failed with status: {}", response.status()),
                });
            return Err(error.message);
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Generic POST request
    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = Request::post(&url).json(body).unwrap();

        // TODO: Add authentication header support for gloo-net 0.6
        // The header API changed in gloo-net 0.6 - needs investigation
        if let Some(_token) = self.get_token() {
            // Authentication headers temporarily disabled
            // Need to use web_sys Headers API or different gloo-net method
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let error: ApiError = response
                .json()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown".to_string(),
                    message: format!("Request failed with status: {}", response.status()),
                });
            return Err(error.message);
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Generic PATCH request
    async fn patch<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = Request::patch(&url).json(body).unwrap();

        // TODO: Add authentication header support for gloo-net 0.6
        // The header API changed in gloo-net 0.6 - needs investigation
        if let Some(_token) = self.get_token() {
            // Authentication headers temporarily disabled
            // Need to use web_sys Headers API or different gloo-net method
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let error: ApiError = response
                .json()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown".to_string(),
                    message: format!("Request failed with status: {}", response.status()),
                });
            return Err(error.message);
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Generic DELETE request
    async fn delete<R: DeserializeOwned>(&self, path: &str) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = Request::delete(&url);

        // TODO: Add authentication header support for gloo-net 0.6
        // The header API changed in gloo-net 0.6 - needs investigation
        if let Some(_token) = self.get_token() {
            // Authentication headers temporarily disabled
            // Need to use web_sys Headers API or different gloo-net method
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let error: ApiError = response
                .json()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown".to_string(),
                    message: format!("Request failed with status: {}", response.status()),
                });
            return Err(error.message);
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // ========================================================================
    // Authentication Endpoints
    // ========================================================================

    pub async fn login(&self, email: String, password: String) -> Result<AuthResponse, String> {
        let request = LoginRequest { email, password };
        let response: AuthResponse = self.post("/api/v1/auth/login", &request).await?;
        self.set_token(&response.access_token);
        Ok(response)
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, String> {
        let request = RegisterRequest {
            username,
            email,
            password,
        };
        let response: AuthResponse = self.post("/api/v1/auth/register", &request).await?;
        self.set_token(&response.access_token);
        Ok(response)
    }

    pub async fn logout(&self) -> Result<(), String> {
        let _: serde_json::Value = self.post("/api/v1/auth/logout", &()).await?;
        self.clear_token();
        Ok(())
    }

    pub async fn refresh_token(&self) -> Result<AuthResponse, String> {
        let response: AuthResponse = self.post("/api/v1/auth/refresh", &()).await?;
        self.set_token(&response.access_token);
        Ok(response)
    }

    // ========================================================================
    // Report Endpoints
    // ========================================================================

    pub async fn create_report(&self, request: CreateReportRequest) -> Result<Report, String> {
        self.post("/api/v1/reports", &request).await
    }

    pub async fn get_reports(&self) -> Result<Vec<Report>, String> {
        self.get("/api/v1/reports").await
    }

    pub async fn get_report(&self, id: &str) -> Result<Report, String> {
        self.get(&format!("/api/v1/reports/{}", id)).await
    }

    pub async fn search_reports(&self, query: &str) -> Result<Vec<Report>, String> {
        self.get(&format!("/api/v1/reports/search?q={}", query))
            .await
    }

    // ========================================================================
    // Alert Endpoints
    // ========================================================================

    pub async fn create_alert(&self, request: CreateAlertRequest) -> Result<Alert, String> {
        self.post("/api/v1/alerts", &request).await
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>, String> {
        self.get("/api/v1/alerts/active").await
    }

    pub async fn get_alert(&self, id: &str) -> Result<Alert, String> {
        self.get(&format!("/api/v1/alerts/{}", id)).await
    }

    pub async fn resolve_alert(&self, id: &str, notes: String) -> Result<Alert, String> {
        #[derive(Serialize)]
        struct ResolveRequest {
            resolution_notes: String,
        }
        self.post(
            &format!("/api/v1/alerts/{}/resolve", id),
            &ResolveRequest {
                resolution_notes: notes,
            },
        )
        .await
    }

    // ========================================================================
    // Story Endpoints
    // ========================================================================

    pub async fn submit_story(&self, request: SubmitStoryRequest) -> Result<Story, String> {
        self.post("/api/v1/stories/submit", &request).await
    }

    pub async fn get_published_stories(&self) -> Result<Vec<Story>, String> {
        self.get("/api/v1/stories").await
    }

    pub async fn get_story(&self, id: &str) -> Result<Story, String> {
        self.get(&format!("/api/v1/stories/{}", id)).await
    }

    // ========================================================================
    // Map Endpoints
    // ========================================================================

    pub async fn get_map_entries(&self, bounds: MapBoundsQuery) -> Result<Vec<MapEntry>, String> {
        self.get(&format!(
            "/api/v1/map/entries?min_lat={}&min_lon={}&max_lat={}&max_lon={}",
            bounds.min_lat, bounds.min_lon, bounds.max_lat, bounds.max_lon
        ))
        .await
    }

    // ========================================================================
    // Published Items Endpoints
    // ========================================================================

    pub async fn get_published_item(&self, slug: &str) -> Result<PublishedItem, String> {
        self.get(&format!("/api/v1/items/{}", slug)).await
    }

    pub async fn get_published_items(&self) -> Result<Vec<PublishedItem>, String> {
        self.get("/api/v1/publish/items").await
    }

    // ========================================================================
    // Admin Endpoints
    // ========================================================================

    pub async fn get_tenants(&self) -> Result<Vec<BusinessTenant>, String> {
        self.get("/api/v1/admin/tenants").await
    }

    pub async fn get_users(&self) -> Result<Vec<UserInfo>, String> {
        self.get("/api/v1/admin/users").await
    }

    // ========================================================================
    // User Profile Endpoints
    // ========================================================================

    pub async fn get_current_user(&self) -> Result<UserInfo, String> {
        self.get("/api/v1/users/me").await
    }

    pub async fn update_profile(&self, username: String, email: String) -> Result<UserInfo, String> {
        #[derive(Serialize)]
        struct UpdateRequest {
            username: String,
            email: String,
        }
        self.patch("/api/v1/users/me", &UpdateRequest { username, email })
            .await
    }
}

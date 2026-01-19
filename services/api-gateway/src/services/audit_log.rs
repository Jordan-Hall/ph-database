use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{AuditLog, User},
};

pub struct AuditService {
    db: Database,
}

impl AuditService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Static method for logging actions - simplified interface
    /// This is a convenience method that doesn't require creating a User object
    pub async fn log_action(
        db: &Database,
        user_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: Option<&str>,
        metadata: Option<serde_json::Value>,
        ip_address: Option<String>,
    ) -> ApiResult<()> {
        let now = Utc::now();

        // Hash IP address for privacy
        let ip_hash = if let Some(ip) = ip_address {
            let mut hasher = Sha256::new();
            hasher.update(ip.as_bytes());
            format!("{:x}", hasher.finalize())
        } else {
            "unknown".to_string()
        };

        let audit_log = AuditLog {
            id: None,
            actor_id: if user_id.starts_with("user:") {
                user_id.to_string()
            } else {
                format!("user:{}", user_id)
            },
            actor_username: user_id.to_string(), // Simplified - just use ID
            actor_ip_hash: ip_hash,
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.unwrap_or("unknown").to_string(),
            details: serde_json::json!({}),
            metadata,
            timestamp: now,
        };

        db.create("audit_log", audit_log).await.map_err(|e| {
            tracing::error!("Failed to create audit log: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create audit log"))
        })?;

        tracing::debug!(
            "Audit log created: {} performed action '{}' on {}:{}",
            user_id,
            action,
            resource_type,
            resource_id.unwrap_or("unknown")
        );

        Ok(())
    }

    /// Create an audit log entry
    pub async fn log(
        &self,
        user: &User,
        action: String,
        resource_type: String,
        resource_id: String,
        details: serde_json::Value,
        metadata: Option<serde_json::Value>,
        ip_address: Option<String>,
    ) -> ApiResult<AuditLog> {
        let now = Utc::now();
        let user_id = user.id.clone().unwrap_or_else(|| "unknown".to_string());

        // Hash IP address for privacy
        let ip_hash = if let Some(ip) = ip_address {
            let mut hasher = Sha256::new();
            hasher.update(ip.as_bytes());
            format!("{:x}", hasher.finalize())
        } else {
            "unknown".to_string()
        };

        let audit_log = AuditLog {
            id: None,
            actor_id: format!("user:{}", user_id),
            actor_username: user.username.clone(),
            actor_ip_hash: ip_hash,
            action,
            resource_type,
            resource_id,
            details,
            metadata,
            timestamp: now,
        };

        let created: Option<AuditLog> = self.db.create("audit_log", audit_log).await.map_err(|e| {
            tracing::error!("Failed to create audit log: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create audit log"))
        })?;

        let log = created.ok_or_else(|| {
            ApiError::Internal(anyhow::anyhow!("Audit log created but not returned"))
        })?;

        tracing::debug!(
            "Audit log created: {} performed action '{}' on {}:{}",
            user.username,
            log.action,
            log.resource_type,
            log.resource_id
        );

        Ok(log)
    }

    /// Get audit logs for a specific user (user can see their own logs)
    pub async fn get_user_logs(
        &self,
        user_id: &str,
        limit: i32,
    ) -> ApiResult<Vec<AuditLog>> {
        let query = format!(
            "SELECT * FROM audit_log WHERE actor_id = user:{} ORDER BY timestamp DESC LIMIT {}",
            user_id, limit
        );

        let mut result = self.db.client.query(&query).await.map_err(|e| {
            tracing::error!("Failed to fetch user audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch user audit logs"))
        })?;

        let logs: Vec<AuditLog> = result.take(0).map_err(|e| {
            tracing::error!("Failed to parse audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse audit logs"))
        })?;

        Ok(logs)
    }

    /// Get all audit logs (admin only)
    pub async fn get_all_logs(
        &self,
        action_filter: Option<String>,
        resource_type_filter: Option<String>,
        limit: i32,
    ) -> ApiResult<Vec<AuditLog>> {
        let mut conditions = vec![];

        if let Some(action) = action_filter {
            conditions.push(format!("action = '{}'", action));
        }

        if let Some(resource_type) = resource_type_filter {
            conditions.push(format!("resource_type = '{}'", resource_type));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            "SELECT * FROM audit_log {} ORDER BY timestamp DESC LIMIT {}",
            where_clause, limit
        );

        let mut result = self.db.client.query(&query).await.map_err(|e| {
            tracing::error!("Failed to fetch audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch audit logs"))
        })?;

        let logs: Vec<AuditLog> = result.take(0).map_err(|e| {
            tracing::error!("Failed to parse audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse audit logs"))
        })?;

        Ok(logs)
    }

    /// Get audit logs for a specific resource
    pub async fn get_resource_logs(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: i32,
    ) -> ApiResult<Vec<AuditLog>> {
        let query = format!(
            "SELECT * FROM audit_log WHERE resource_type = '{}' AND resource_id = '{}' ORDER BY timestamp DESC LIMIT {}",
            resource_type, resource_id, limit
        );

        let mut result = self.db.client.query(&query).await.map_err(|e| {
            tracing::error!("Failed to fetch resource audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to fetch resource audit logs"))
        })?;

        let logs: Vec<AuditLog> = result.take(0).map_err(|e| {
            tracing::error!("Failed to parse audit logs: {}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to parse audit logs"))
        })?;

        Ok(logs)
    }
}

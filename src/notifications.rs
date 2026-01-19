use dioxus::prelude::*;
use std::collections::VecDeque;
use web_sys::{Notification, NotificationPermission};

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToastNotification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub duration_ms: u32,
}

/// Global notification state
#[derive(Clone, Copy)]
pub struct NotificationService {
    notifications: Signal<VecDeque<ToastNotification>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notifications: Signal::new(VecDeque::new()),
        }
    }

    /// Add a success notification
    pub fn success(&mut self, message: impl Into<String>) {
        self.add_notification(message.into(), NotificationType::Success, 5000);
    }

    /// Add an error notification
    pub fn error(&mut self, message: impl Into<String>) {
        self.add_notification(message.into(), NotificationType::Error, 7000);
    }

    /// Add a warning notification
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add_notification(message.into(), NotificationType::Warning, 6000);
    }

    /// Add an info notification
    pub fn info(&mut self, message: impl Into<String>) {
        self.add_notification(message.into(), NotificationType::Info, 5000);
    }

    /// Add a notification to the queue
    fn add_notification(&mut self, message: String, notification_type: NotificationType, duration_ms: u32) {
        let id = uuid::Uuid::new_v4().to_string();
        let notification = ToastNotification {
            id: id.clone(),
            message,
            notification_type,
            duration_ms,
        };

        self.notifications.write().push_back(notification);

        // Auto-remove after duration
        let notifications = self.notifications;
        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(duration_ms as u64)).await;
            notifications.write().retain(|n| n.id != id);
        });
    }

    /// Remove a notification by ID
    pub fn remove(&mut self, id: &str) {
        self.notifications.write().retain(|n| n.id != id);
    }

    /// Get all current notifications
    pub fn get_notifications(&self) -> Vec<ToastNotification> {
        self.notifications.read().iter().cloned().collect()
    }

    /// Request browser notification permission
    pub async fn request_permission() -> bool {
        if let Ok(Some(window)) = web_sys::window().ok_or("No window") {
            if let Ok(promise) = Notification::request_permission(&window) {
                if let Ok(result) = wasm_bindgen_futures::JsFuture::from(promise).await {
                    if let Some(permission) = result.as_string() {
                        return permission == "granted";
                    }
                }
            }
        }
        false
    }

    /// Check if browser notifications are supported and permitted
    pub fn has_permission() -> bool {
        if let Some(permission) = Notification::permission() {
            permission == NotificationPermission::Granted
        } else {
            false
        }
    }

    /// Show a browser notification
    pub fn show_browser_notification(title: &str, body: &str) {
        if Self::has_permission() {
            if let Ok(notification) = Notification::new_with_options(
                title,
                web_sys::NotificationOptions::new().body(body),
            ) {
                log::debug!("Browser notification shown: {}", title);

                // Auto-close after 5 seconds
                spawn(async move {
                    gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
                    notification.close();
                });
            }
        }
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Toast notification component
#[component]
pub fn NotificationContainer() -> Element {
    let mut notification_service = use_context::<NotificationService>();
    let notifications = notification_service.get_notifications();

    rsx! {
        div { class: "notification-container",
            style: "position: fixed; top: 20px; right: 20px; z-index: 9999; max-width: 400px;",
            for notification in notifications {
                ToastNotification {
                    key: "{notification.id}",
                    notification: notification.clone(),
                    on_close: move |_| {
                        notification_service.remove(&notification.id);
                    }
                }
            }
        }
    }
}

#[component]
fn ToastNotification(
    notification: ToastNotification,
    on_close: EventHandler<()>,
) -> Element {
    let (bg_color, border_color, icon) = match notification.notification_type {
        NotificationType::Success => ("#d4edda", "#c3e6cb", "✓"),
        NotificationType::Error => ("#f8d7da", "#f5c6cb", "✕"),
        NotificationType::Warning => ("#fff3cd", "#ffeaa7", "⚠"),
        NotificationType::Info => ("#d1ecf1", "#bee5eb", "ℹ"),
    };

    rsx! {
        div {
            class: "govuk-notification-banner",
            role: "alert",
            style: "
                background-color: {bg_color};
                border: 2px solid {border_color};
                margin-bottom: 10px;
                padding: 15px;
                border-radius: 4px;
                box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                animation: slideIn 0.3s ease-out;
            ",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                div { style: "display: flex; align-items: center; gap: 10px;",
                    span { style: "font-size: 20px; font-weight: bold;", "{icon}" }
                    p { class: "govuk-body", style: "margin: 0;", "{notification.message}" }
                }
                button {
                    class: "govuk-button govuk-button--secondary",
                    style: "
                        background: transparent;
                        border: none;
                        font-size: 20px;
                        cursor: pointer;
                        padding: 0;
                        margin: 0;
                        min-width: auto;
                    ",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
        }
    }
}

/// CSS animation for toast
const _ANIMATION_CSS: &str = r#"
@keyframes slideIn {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}
"#;

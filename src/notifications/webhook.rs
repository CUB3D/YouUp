use crate::data::webhook_subscription_repository::WebhookSubscriberRepository;
use reqwest::Client;
use serde::Serialize;

pub struct WebhookNotifier {
    client: Client,
}

impl Default for WebhookNotifier {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl WebhookNotifier {
    pub async fn notify_all_subscribers(
        &self,
        subscribers: &WebhookSubscriberRepository,
        message: WebhookPayload,
    ) {
        let _span = tracing::info_span!("Calling webhooks");

        for sub in subscribers.get_all_enabled_subscribers() {
            match self.client.post(&sub.url).json(&message).send().await {
                Ok(_) => {}
                Err(e) => tracing::warn!("Failed to call webhook {}: {:?}", &sub.url, e),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub project_id: i32,
    pub project_name: String,
    pub status_code: u16,
    pub time: String,
}

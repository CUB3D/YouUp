use crate::data::sms_subscription_repository::SmsSubscriberRepository;
use crate::settings;
use twilio::{Client, OutboundMessage};

pub struct SMSNotifier {
    client: Client,
}

impl Default for SMSNotifier {
    fn default() -> Self {
        Self {
            client: Client::new(
                &settings::twilio_account_id(),
                &settings::twilio_auth_token(),
            ),
        }
    }
}

impl SMSNotifier {
    pub async fn send_message(&self, dest: &str, message: &str) {
        let _span = tracing::info_span!("Sending SMS message");

        if !settings::sms_enabled() {
            tracing::warn!("SMS not enabled, not sending SMS notifications");
            return;
        }

        let res = self
            .client
            .send_message(OutboundMessage::new(
                &settings::twilio_contact_number(),
                dest,
                message,
            ))
            .await;
        tracing::info!("Send SMS message: {:?}", res);
    }

    pub async fn notify_all_subscribers(
        &self,
        sms_subscribers_repo: &SmsSubscriberRepository,
        message: &str,
    ) {
        for sub in sms_subscribers_repo.get_all_confirmed_subscribers() {
            self.send_message(&sub.phone_number, message).await;
        }
    }
}

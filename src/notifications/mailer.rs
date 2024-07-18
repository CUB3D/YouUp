use crate::db::Database;
use crate::diesel::RunQueryDsl;
use crate::models::EmailSubscription;
use crate::settings;
use diesel::{ExpressionMethods, QueryDsl};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct Mailer {
    creds: Credentials,
}

impl Default for Mailer {
    fn default() -> Self {
        Self {
            creds: Credentials::new(settings::smtp_username(), settings::smtp_password()),
        }
    }
}

impl Mailer {
    pub fn send_message(&self, email: Message) {
        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(self.creds.clone())
            .build();

        // Send the email
        let result = mailer.send(&email);

        if result.is_ok() {
            info!("Email sent");
        } else {
            warn!("Could not send email: {:?}", result);
        }
    }

    pub fn send_to_subscribers(
        &self,
        db: &Database,
        from: &str,
        title: String,
        message_body: String,
    ) {
        use crate::schema::email_subscriptions;

        let subscribed_users = email_subscriptions::table
            .filter(email_subscriptions::dsl::confirmed.eq(true))
            .load::<EmailSubscription>(&mut db.get().unwrap())
            .unwrap();

        for user in &subscribed_users {
            let email = Message::builder()
                .to(user.email.parse().unwrap())
                .from(from.parse().unwrap())
                .header(ContentType::TEXT_HTML)
                .subject(&title)
                .body(message_body.to_string())
                .unwrap();

            self.send_message(email);
        }
    }
}

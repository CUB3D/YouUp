use crate::settings;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use crate::db::Database;
use diesel::{QueryDsl, ExpressionMethods};
use crate::models::EmailSubscription;
use crate::diesel::RunQueryDsl;

#[derive(Clone)]
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
            println!("Email sent");
        } else {
            println!("Could not send email: {:?}", result);
        }
    }

    pub fn send_to_subscribers(&self, db: &Database, from: &str, title: String, message_body: &str) {
        use crate::schema::email_subscriptions;

        let subscribed_users = email_subscriptions::table.filter(email_subscriptions::dsl::confirmed.eq(true))
            .load::<EmailSubscription>(&db.get().unwrap())
            .unwrap();

        for user in &subscribed_users {
            let email = Message::builder()
                // Addresses can be specified by the tuple (email, alias)
                .to(user.email.parse().unwrap())
                .from(from.parse().unwrap())
                .subject(&title)
                .body(message_body.clone())
                .unwrap();

            self.send_message(email);
        }
    }
}

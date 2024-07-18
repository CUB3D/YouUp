use crate::db::Database;
use crate::models::WebhookSubscription;
use crate::schema::webhook_subscriptions;
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub type WebhookSubscriberRepository = Box<dyn WebhookSubscriptionRepository>;
pub type WebhookSubscriberRepositoryData = Data<WebhookSubscriberRepository>;

pub trait WebhookSubscriptionRepository {
    fn get_all_enabled_subscribers(&self) -> Vec<WebhookSubscription>;
    fn get_all(&self) -> Vec<WebhookSubscription>;
}

impl WebhookSubscriptionRepository for Database {
    fn get_all_enabled_subscribers(&self) -> Vec<WebhookSubscription> {
        webhook_subscriptions::table
            .filter(webhook_subscriptions::enabled.eq(true))
            .load::<WebhookSubscription>(&mut self.get().unwrap())
            .expect("Unable to load webhook subscribers")
    }

    fn get_all(&self) -> Vec<WebhookSubscription> {
        webhook_subscriptions::table
            .load::<WebhookSubscription>(&mut self.get().unwrap())
            .expect("Unable to load webhook subscribers")
    }
}

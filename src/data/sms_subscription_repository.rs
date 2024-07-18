use crate::db::Database;
use crate::models::SmsSubscription;
use crate::schema::sms_subscriptions;
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

pub type SmsSubscriberRepository = Box<dyn SmsSubscriptionRepository>;
pub type SmsSubscriberRepositoryData = Data<SmsSubscriberRepository>;

pub trait SmsSubscriptionRepository {
    fn get_all_confirmed_subscribers(&self) -> Vec<SmsSubscription>;
    fn get_all(&self) -> Vec<SmsSubscription>;
}

impl SmsSubscriptionRepository for Database {
    fn get_all_confirmed_subscribers(&self) -> Vec<SmsSubscription> {
        sms_subscriptions::table
            .filter(sms_subscriptions::confirmed.eq(true))
            .load::<SmsSubscription>(&mut self.get().unwrap())
            .expect("Unable to load sms subscribers")
    }

    fn get_all(&self) -> Vec<SmsSubscription> {
        sms_subscriptions::table
            .load::<SmsSubscription>(&mut self.get().unwrap())
            .expect("Unable to load sms subscribers")
    }
}

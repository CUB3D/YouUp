use crate::db::Database;
use crate::models::{EmailSubscription, NewEmailSubscription};
use crate::notifications::mailer::Mailer;
use actix_web::get;
use actix_web::post;
use actix_web::web::{Data, Query};
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use lettre::message::Mailbox;
use lettre::Message;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "email_confirm_subscription.html")]
pub struct EmailSubscriptionTemplate {
    pub site_url: String,
    pub subscription_id: String,
}

#[derive(Deserialize)]
pub struct EmailSubscribeRequest {
    pub email: String,
}

#[post("/subscribe/email")]
pub async fn post_email_subscribe(
    db: Data<Database>,
    mailer: Data<Arc<Mailer>>,
    form: Form<EmailSubscribeRequest>,
) -> impl Responder {
    use crate::schema::email_subscriptions;

    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Email subscribe", request_id = %request_id);
    let _span_guard = span.enter();

    let parsed_email = form.email.clone().parse::<Mailbox>();

    if let Ok(email) = parsed_email {
        diesel::insert_into(email_subscriptions::table)
            .values(NewEmailSubscription {
                email: form.email.clone(),
            })
            .execute(&mut db.get().unwrap())
            .unwrap();

        let entry = email_subscriptions::table
            .filter(email_subscriptions::dsl::email.eq(form.email.clone()))
            .load::<EmailSubscription>(&mut db.get().unwrap())
            .unwrap();

        let message_body = EmailSubscriptionTemplate {
            site_url: "localhost:8102".to_string(),
            subscription_id: format!("{}", entry.first().unwrap().id),
        }
        .render()
        .unwrap();

        let email = Message::builder()
            // Addresses can be specified by the tuple (email, alias)
            .to(email)
            .from("YouUp <subscriptions@you-up.net>".parse().unwrap())
            .subject("Confirm your subscription")
            .body(message_body)
            .unwrap();

        mailer.send_message(email);

        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
}

//TODO: this should use a random token
#[derive(Deserialize)]
pub struct ConfirmSubscription {
    id: i32,
}

#[get("/subscribe/email/confirm")]
pub async fn get_email_confirm(
    db: Data<Database>,
    form: Query<ConfirmSubscription>,
) -> impl Responder {
    use crate::schema::email_subscriptions;

    //TODO: add a tracking id (uuid?)
    let span = tracing::info_span!("Email subscribe confirm, id = {}", form.id);
    let _span_guard = span.enter();

    diesel::update(email_subscriptions::table.filter(email_subscriptions::dsl::id.eq(form.id)))
        .set(email_subscriptions::dsl::confirmed.eq(true))
        .execute(&mut db.get().unwrap())
        .unwrap();

    tracing::info!("Confirmed subscription id={}", form.id);

    HttpResponse::Ok()
}

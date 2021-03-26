use crate::data::sms_subscription_repository::SmsSubscriberRepositoryData;
use crate::data::webhook_subscription_repository::WebhookSubscriberRepositoryData;
use crate::db::Database;
use crate::models::{EmailSubscription, SmsSubscription, WebhookSubscription};
use crate::schema::email_subscriptions::dsl::email_subscriptions;
use crate::settings;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use diesel::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin_subscriptions.html")]
pub struct AdminSubscriptionTemplate {
    pub sms_enabled: bool,
    pub subscriptions: Vec<EmailSubscription>,
    pub sms_subscriptions: Vec<SmsSubscription>,
    pub webhook_subscriptions: Vec<WebhookSubscription>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize)]
pub struct ProjectUpdate {}

async fn admin_subscription(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    sms_subscriptions_repo: SmsSubscriberRepositoryData,
    webhook_subscriptions_repo: WebhookSubscriberRepositoryData,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin subscription", request_id = %request_id);
    let _guard = span.enter();

    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin")
            .finish();
    }

    let subscriptions = email_subscriptions
        .load::<EmailSubscription>(&pool.get().unwrap())
        .expect("Unable to load subscriptions");

    let template = AdminSubscriptionTemplate {
        subscriptions,
        sms_subscriptions: sms_subscriptions_repo.get_all(),
        webhook_subscriptions: webhook_subscriptions_repo.get_all(),
        sms_enabled: settings::sms_enabled(),
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/subscriptions")]
pub async fn get_admin_subscriptions(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    sms_subscriptions_repo: SmsSubscriberRepositoryData,
    webhook_subscriptions_repo: WebhookSubscriberRepositoryData,
) -> impl Responder {
    admin_subscription(
        id,
        pool,
        settings,
        sms_subscriptions_repo,
        webhook_subscriptions_repo,
    )
    .await
}

//TODO: is this needed
#[post("/admin/subscriptions")]
pub async fn post_admin_subscriptions(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    _updates: Option<Form<ProjectUpdate>>,
    sms_subscriptions_repo: SmsSubscriberRepositoryData,
    webhook_subscriptions_repo: WebhookSubscriberRepositoryData,
) -> impl Responder {
    admin_subscription(
        id,
        pool,
        settings,
        sms_subscriptions_repo,
        webhook_subscriptions_repo,
    )
    .await
}

use crate::db::Database;
use crate::models::EmailSubscription;
use crate::schema::email_subscriptions::dsl::email_subscriptions;
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
    pub subscriptions: Vec<EmailSubscription>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize)]
pub struct ProjectUpdate {}

async fn admin_subscription(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
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
) -> impl Responder {
    admin_subscription(id, pool, settings).await
}

//TODO: is this needed
#[post("/admin/subscriptions")]
pub async fn post_admin_subscriptions(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    _updates: Option<Form<ProjectUpdate>>,
) -> impl Responder {
    admin_subscription(id, pool, settings).await
}

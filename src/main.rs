use actix_files::Files;
use actix_rt::spawn;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::middleware::{Compress, Logger, NormalizePath, TrailingSlash};
use actix_web::web::Data;
use dotenv::dotenv;

use crate::data::incident_repository::IncidentRepository;
use crate::data::project_repository::ProjectRepository;
use crate::data::sms_subscription_repository::SmsSubscriptionRepository;
use crate::data::status_repository::StatusRepository;
use crate::data::webhook_subscription_repository::WebhookSubscriptionRepository;
use crate::form_email_subscribe::{get_email_confirm, post_email_subscribe};
use crate::notifications::mailer::Mailer;
use crate::notifications::sms::SMSNotifier;
use crate::notifications::webhook::WebhookNotifier;
use crate::settings::PersistedSettings;
use crate::template::index::status_day::StatusDay;
use crate::template::index::template_index::{get_index, head_index};
use crate::template::template_admin_dashboard::{get_admin_dashboard, post_admin_dashboard};
use crate::template::template_admin_incident::get_admin_incidents;
use crate::template::template_admin_incident_new::{
    get_admin_incidents_new, post_admin_incidents_new,
};
use crate::template::template_admin_incident_status_new::{
    get_admin_incident_status_new, post_admin_incident_status_new,
};
use crate::template::template_admin_login::{get_admin_login, post_admin_login};
use crate::template::template_admin_subscriptions::{
    get_admin_subscriptions, post_admin_subscriptions,
};
use crate::template::template_embed::get_embed;
use crate::template::template_feed_atom::get_atom_feed;
use crate::template::template_feed_rss::get_rss_feed;
use crate::template::template_history::get_incident_history;
use crate::template::template_incident_details::get_incident_details;
use crate::template::template_uptime::get_uptime;
use crate::update_job::{process_pending_status_updates_job, run_update_job};
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::config::CookieContentSecurity;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::{Key, SameSite};
use sentry_tracing::EventFilter;
use std::env;
use std::sync::Arc;
use template::template_admin_project_new::get_admin_project_new;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod data;
pub mod db;
pub mod form_email_subscribe;
pub mod models;
pub mod notifications;
pub mod project_status;
pub mod schema;
pub mod settings;
pub mod template;
pub mod time_formatter;
pub mod time_utils;
pub mod update_job;
pub mod utils;
//TODO: REST API
//TODO: twitter
//TODO: make sentry optional

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let _guard = sentry::init((
        "https://c2d3ab1d150243ce9a828d92d5a77452@o289707.ingest.sentry.io/6486846",
        sentry::ClientOptions {
            // Set this to a lower value in production
            traces_sample_rate: 1.0,
            release: sentry::release_name!(),
            ..sentry::ClientOptions::default()
        },
    ));

    let sentry_layer = sentry_tracing::layer().event_filter(|md| match md.level() {
        &tracing::Level::ERROR => EventFilter::Event,
        _ => EventFilter::Ignore,
    });

    {
        let temp = tracing_subscriber::fmt::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .finish();

        if settings::sentry_enabled() {
            temp.with(sentry_layer).init();
        } else {
            temp.init();
        }
    }

    let db = db::get_db_connection().expect("Failed to get DB");
    let mailer = Arc::new(Mailer::default());
    let sms = Arc::new(SMSNotifier::default());
    let webhook = Arc::new(WebhookNotifier::default());

    if env::var("UPDATE").unwrap_or_else(|_| "1".to_string()) == "1" {
        spawn(run_update_job(
            mailer.clone(),
            sms.clone(),
            webhook.clone(),
            db.clone(),
            Box::new(db.clone()) as Box<dyn SmsSubscriptionRepository>,
            Box::new(db.clone()) as Box<dyn WebhookSubscriptionRepository>,
        ));
        spawn(process_pending_status_updates_job(db.clone()));
    }

    let host = settings::get_host_domain();

    tracing::info!("Running on http://{}", host);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(Box::new(db.clone()) as Box<dyn ProjectRepository>))
            .app_data(Data::new(
                Box::new(db.clone()) as Box<dyn IncidentRepository>
            ))
            .app_data(Data::new(Box::new(db.clone()) as Box<dyn StatusRepository>))
            .app_data(Data::new(
                Box::new(db.clone()) as Box<dyn SmsSubscriptionRepository>
            ))
            .app_data(Data::new(
                Box::new(db.clone()) as Box<dyn WebhookSubscriptionRepository>
            ))
            .app_data(Data::new(PersistedSettings::new(db.clone())))
            .app_data(Data::new(mailer.clone()))
            .app_data(Data::new(sms.clone()))
            .app_data(Data::new(webhook.clone()))
            .service(Files::new("/static", "./static"))
            .service(get_index)
            .service(head_index)
            .service(get_uptime)
            .service(get_incident_details)
            .service(get_admin_login)
            .service(post_admin_login)
            .service(get_rss_feed)
            .service(get_atom_feed)
            .service(get_incident_history)
            .service(get_embed)
            .service(get_admin_dashboard)
            .service(post_admin_dashboard)
            .service(get_admin_subscriptions)
            .service(post_admin_subscriptions)
            .service(post_email_subscribe)
            .service(get_admin_incidents)
            .service(get_email_confirm)
            .service(get_admin_incidents_new)
            .service(post_admin_incidents_new)
            .service(get_admin_incident_status_new)
            .service(post_admin_incident_status_new)
            .service(get_admin_project_new)
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(IdentityMiddleware::default())
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&settings::private_key()),
                )
                .cookie_name("you-up-auth".to_string())
                .cookie_secure(!settings::insecure())
                .cookie_content_security(CookieContentSecurity::Private)
                .cookie_same_site(SameSite::Strict)
                .build(),
            )
    })
    .bind(host)?
    .run()
    .await
}

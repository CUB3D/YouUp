use crate::models::{NewStatus, Project, Status};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use http::status::StatusCode;
use reqwest::Client;

use crate::data::sms_subscription_repository::SmsSubscriberRepository;
use crate::data::webhook_subscription_repository::WebhookSubscriberRepository;
use crate::db::Database;
use crate::notifications::mailer::Mailer;
use crate::notifications::sms::SMSNotifier;
use crate::notifications::webhook::{WebhookNotifier, WebhookPayload};
use crate::schema::status as stat;
use chrono::Utc;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

lazy_static! {
    static ref PENDING_STATUS_UPDATES: Mutex<Vec<NewStatus>> = Mutex::new(Vec::new());
}

#[tracing::instrument(skip(db))]
pub fn submit_status(db: Database, status: NewStatus) {
    match diesel::insert_into(stat::table)
        .values(status.clone())
        .execute(&db.get().unwrap())
    {
        Ok(_) => {}
        Err(_) => {
            if let Ok(mut lock) = PENDING_STATUS_UPDATES.lock() {
                lock.deref_mut().push(status)
            }
        }
    }
}

#[tracing::instrument(skip(db))]
pub async fn process_pending_status_updates_job(db: Database) {
    let _span = tracing::info_span!("Process pending updates job");

    loop {
        if let Ok(mut lock) = PENDING_STATUS_UPDATES.lock() {
            let status = lock.deref().first();
            if let Some(status) = status {
                if diesel::insert_into(stat::table)
                    .values(status)
                    .execute(&db.get().unwrap())
                    .is_ok()
                {
                    lock.deref_mut().remove(0);
                }
            }
        }

        actix_rt::time::sleep(Duration::from_secs(90)).await;
    }
}

#[tracing::instrument(skip(db, webhook_subscription_repo, sms_subscription_repo))]
pub async fn run_update_job(
    mailer: Arc<Mailer>,
    sms: Arc<SMSNotifier>,
    webhook: Arc<WebhookNotifier>,
    db: Database,
    sms_subscription_repo: SmsSubscriberRepository,
    webhook_subscription_repo: WebhookSubscriberRepository,
) {
    let _span = tracing::info_span!("Update Job");

    let c = Client::builder().build().unwrap();

    loop {
        let projects_list = crate::schema::projects::dsl::projects
            .load::<Project>(&db.get().unwrap())
            .expect("Unable to load projects");

        for domain in &projects_list {
            tracing::info!("Checking {}", domain.name);

            // Check if domain is up, store in db and wait
            let req = c.get(&domain.url).send();
            let req_start_time = Instant::now();
            let response = req.await;
            let req_duration = req_start_time.elapsed();
            let status = response
                .map(|v| v.status())
                .unwrap_or(StatusCode::NOT_FOUND);

            // Get the most recent status
            let most_recent_status = stat::table
                .filter(stat::dsl::project.eq(domain.id))
                .order_by(stat::dsl::created.desc())
                .limit(1)
                .load::<Status>(&db.get().unwrap());

            submit_status(
                db.clone(),
                NewStatus {
                    project: domain.id,
                    //TODO: change the type of this field
                    time: req_duration.as_millis() as i32,
                    status_code: status.as_u16() as i32,
                },
            );

            if let Ok(stat) = most_recent_status {
                if let Some(stat2) = stat.first() {
                    if stat2.is_success() && !status.is_success() {
                        mailer.send_to_subscribers(
                            &db,
                            "YouUp <alerts@you-up.net>",
                            format!("Alert in project '{}'", domain.name),
                            format!(
                                "Service is now down, received a status code of {} at {}",
                                status.as_str(),
                                Utc::now().format("%+")
                            ),
                        );

                        sms.notify_all_subscribers(
                            &sms_subscription_repo,
                            &format!(
                                "YouUp, Project '{}' down, code {}",
                                domain.name,
                                status.as_str()
                            ),
                        )
                        .await;

                        webhook
                            .notify_all_subscribers(
                                &webhook_subscription_repo,
                                WebhookPayload {
                                    project_id: domain.id,
                                    project_name: domain.name.clone(),
                                    status_code: status.as_u16(),
                                    time: Utc::now().format("%+").to_string(),
                                },
                            )
                            .await;
                    }
                }
            }
        }

        actix_rt::time::sleep(Duration::from_secs(90)).await;
    }
}

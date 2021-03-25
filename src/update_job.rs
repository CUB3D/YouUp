use crate::models::{NewStatus, Project, Status};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use http::status::StatusCode;
use reqwest::Client;

use crate::data::sms_subscription_repository::SmsSubscriberRepository;
use crate::db::Database;
use crate::notifications::mailer::Mailer;
use crate::notifications::sms::SMSNotifier;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub async fn run_update_job(
    mailer: Arc<Mailer>,
    sms: Arc<SMSNotifier>,
    db: Database,
    sms_subscription_repo: SmsSubscriberRepository,
) {
    let c = Client::builder().build().unwrap();

    loop {
        use crate::schema::projects::dsl::*;
        use crate::schema::status as stat;

        let projects_list = projects
            .load::<Project>(&db.get().unwrap())
            .expect("Unable to load projects");

        for domain in &projects_list {
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

            diesel::insert_into(stat::table)
                .values(NewStatus {
                    project: domain.id,
                    //TODO: change the type of this field
                    time: req_duration.as_millis() as i32,
                    status_code: status.as_u16() as i32,
                })
                .execute(&db.get().unwrap())
                .unwrap();

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
                    }
                }
            }
        }

        actix_rt::time::delay_for(Duration::from_secs(90)).await;
    }
}

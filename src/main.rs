use crate::models::{IncidentStatusType, IncidentStatusUpdate, Incidents, Project, Status};
use actix_files::Files;
use actix_rt::spawn;

use crate::diesel::GroupedBy;
use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::web::{resource, Data};
use actix_web::HttpServer;
use actix_web::{App, HttpResponse, Responder};
use askama::Template;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use dotenv::dotenv;

use crate::db::Database;
use crate::diesel::BelongingToDsl;
use crate::form_email_subscribe::{get_email_confirm, post_email_subscribe};
use crate::mailer::Mailer;
use crate::project_status::ProjectStatusTypes;
use crate::settings::{PersistedSettings, CUSTOM_HTML, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template_admin_dashboard::get_admin_dashboard;
use crate::template_admin_dashboard::post_admin_dashboard;
use crate::template_admin_login::post_admin_login;
use crate::template_admin_subscriptions::{get_admin_subscriptions, post_admin_subscriptions};
use crate::template_index::IndexTemplate;
use crate::template_tooltip::StatusTooltipTemplate;
use crate::update_job::run_update_job;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use diesel::expression::sql_literal::sql;
use env_logger::Env;
use std::convert::TryInto;
use std::env;
use std::ops::Sub;
use std::sync::Arc;
use template_admin_login::get_admin_login;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod db;
pub mod form_email_subscribe;
pub mod mailer;
pub mod models;
pub mod project_status;
pub mod schema;
pub mod settings;
pub mod template_admin_dashboard;
pub mod template_admin_login;
pub mod template_admin_subscriptions;
pub mod template_index;
pub mod template_tooltip;
pub mod time_formatter;
pub mod update_job;

pub struct IncidentDay {
    pub date: String,
    pub incidents: Vec<(Incidents, Vec<(IncidentStatusUpdate, IncidentStatusType)>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Downtime {
    pub duration: String,
}

#[derive(Clone)]
pub struct StatusDay {
    pub status: Vec<Status>,
    pub date: String,
    pub downtime: Vec<Downtime>,
}

impl StatusDay {
    fn get_overall_status(&self) -> ProjectStatusTypes {
        if self.status.is_empty() {
            return ProjectStatusTypes::Unknown;
        }
        if self.downtime.is_empty() && !self.status.is_empty() {
            return ProjectStatusTypes::Operational;
        }

        let initial_status = if self.status.iter().all(|x| !x.is_success()) {
            ProjectStatusTypes::Failed
        } else {
            //TODO: apply more logic here, should really only be failed if more fail than success (based on amount of time)
            ProjectStatusTypes::Failing
        };

        // If a system was failing and is now working again show it as recovering to indicate this
        if initial_status == ProjectStatusTypes::Failing
            && self.status.last().map(|s| s.is_success()).unwrap_or(false)
        {
            ProjectStatusTypes::Recovering
        } else {
            initial_status
        }
    }

    fn get_tooltip(&self) -> String {
        StatusTooltipTemplate { day: self.clone() }
            .render()
            .expect("Unable to render tooltip")
    }

    fn avg_request_time(&self) -> u32 {
        if self.status.is_empty() {
            0
        } else {
            self.status.iter().map(|s| s.time).sum::<i32>() as u32 / (self.status.len() as u32)
        }
    }

    fn get_chart_status(&self) -> &[Status] {
        self.status.as_slice() //[max(self.status.len() - 100, 0usize)..self.status.len()]
    }
}

pub struct ProjectStatus {
    pub project: Project,
    pub days: Vec<StatusDay>,
    pub today: StatusDay,
}

#[cfg(test)]
mod test {
    use crate::compute_downtime_periods;
    use crate::models::Status;
    use chrono::{TimeZone, Utc};
    use std::ops::Sub;

    #[actix_rt::test]
    async fn compute_simple_downtime() {
        let x = compute_downtime_periods(&[Status {
            created: Utc::now().naive_utc(),
            status_code: 200,
            id: 0,
            project: 0,
            time: 0,
        }])
        .await;

        assert!(x.is_empty())
    }

    #[actix_rt::test]
    async fn compute_downtime() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc::now().naive_utc(),
                status_code: 200,
                id: 3,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc::now().naive_utc().sub(chrono::Duration::hours(1)),
                status_code: 503,
                id: 2,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc::now().naive_utc().sub(chrono::Duration::hours(2)),
                status_code: 200,
                id: 1,
                project: 0,
                time: 10,
            },
        ])
        .await;

        assert_eq!(x.first().unwrap().duration, "59 minutes");
        assert_eq!(x.len(), 1);
    }

    #[actix_rt::test]
    async fn compute_downtime_end_of_day() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(23, 0, 0).naive_utc(),
                // Utc::now().naive_utc().sub(chrono::Duration::hours(1)),
                status_code: 200,
                id: 2,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(1, 0, 0).naive_utc(),
                status_code: 404,
                id: 1,
                project: 0,
                time: 10,
            },
        ])
        .await;

        assert_eq!(x.first().unwrap().duration, "23 hours");
        assert_eq!(x.len(), 1);
    }

    #[actix_rt::test]
    async fn compute_downtime_never_up() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(0, 0, 0).naive_utc(),
                status_code: 404,
                id: 1,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(12, 0, 0).naive_utc(),
                status_code: 404,
                id: 2,
                project: 0,
                time: 10,
            },
        ])
        .await;

        assert_eq!(x.first().unwrap().duration, "24 hours");
        assert_eq!(x.len(), 1);
    }
}

async fn compute_downtime_periods(status_on_day: &[Status]) -> Vec<Downtime> {
    if !status_on_day.is_empty() && status_on_day.iter().all(|s| !s.is_success()) {
        let first_day = status_on_day.first().unwrap().created;

        // Get 00:00:00 on the next day
        let end_of_first_day = first_day
            .checked_add_signed(Duration::days(1))
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        // Find the duration between the last point that we have info about and the first point
        // If the end of the day is in the future, then this downtime occurs today, and we will state that it is down until now
        // If this day is in the past then we will assume it was down for the rest of the day
        //TODO: we might want to check when it's up the next day
        // also might want to look back to before the first result of the day?
        let duration = Utc::now()
            .naive_utc()
            .min(end_of_first_day)
            .signed_duration_since(first_day);

        return vec![Downtime {
            duration: time_formatter::format_duration(&duration),
        }];
    }

    let mut downtime = Vec::new();
    let mut downtime_period_start: Option<NaiveDateTime> = status_on_day
        .first()
        .map(|e| Some(e.created))
        .unwrap_or(None);

    for item in status_on_day.iter() {
        if item.is_success() {
            if let Some(tmp) = downtime_period_start {
                let period_duration = tmp.signed_duration_since(item.created);

                if period_duration.num_minutes() > settings::get_minimum_downtime_minutes() {
                    downtime.push(Downtime {
                        duration: time_formatter::format_duration(&period_duration),
                    });
                }
                downtime_period_start = None;
            }
        } else {
            // If we are currently down then we will skip until we find the next up
            if downtime_period_start.is_none() {
                downtime_period_start = Some(item.created)
            }
        }
    }
    if let Some(tmp) = downtime_period_start {
        // If it was still down at the last reading of the day then assume it was down for all of the rest of that day

        let time_of_first_request = status_on_day.first().map(|s| s.created).unwrap();

        let end_of_day = time_of_first_request
            .checked_add_signed(Duration::days(1))
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        // If this day was in the past then take its end_of_day however if we are considering today then define the end to be the current time
        let clamped_end_of_day = end_of_day.min(Utc::now().naive_utc());

        let period_duration = clamped_end_of_day.signed_duration_since(tmp);
        if period_duration.num_minutes() > settings::get_minimum_downtime_minutes() {
            downtime.push(Downtime {
                duration: time_formatter::format_duration(&period_duration),
            });
        }
    }

    downtime
}

pub async fn root(pool: Data<Database>, settings: Data<PersistedSettings>) -> impl Responder {
    use self::schema::projects::dsl::*;
    use self::schema::status as stat;

    let projects_list = projects
        .load::<Project>(&pool.get().unwrap())
        .expect("Unable to load projects");

    let status_list: Vec<_> = stat::dsl::status
        .filter(sql("created > DATE_SUB(NOW(), INTERVAL 30 day)"))
        .order(stat::dsl::created.desc())
        .load::<Status>(&pool.get().unwrap())
        .expect("Unable to load status");

    let history_size = settings::get_history_size();

    let mut p = Vec::new();
    for proj in projects_list {
        let mut days: Vec<StatusDay> = Vec::with_capacity(history_size);
        for x in (0..history_size).rev() {
            let now = Utc::now();
            let then = now
                .sub(chrono::Duration::days(x.try_into().unwrap()))
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .naive_utc();

            let status_on_day: Vec<_> = status_list
                .iter()
                .filter(|s| s.project == proj.id && s.created.date() == then.date())
                .cloned()
                .collect();

            let downtime = compute_downtime_periods(status_on_day.as_slice()).await;

            days.push(StatusDay {
                status: status_on_day,
                date: then.format("%Y/%m/%d").to_string(),
                downtime,
            });
        }

        let today = days.last().unwrap().clone();

        p.push(ProjectStatus {
            project: proj,
            days,
            today,
        })
    }

    use self::schema::incident_status_type;
    use self::schema::incident_status_update;
    use self::schema::incidents::dsl::*;

    let mut incident_days = Vec::with_capacity(10);

    let all_incidents: Vec<_> = incidents
        .filter(sql("created > DATE_SUB(NOW(), INTERVAL 10 day)"))
        .load::<Incidents>(&pool.get().unwrap())
        .unwrap();

    let incident_days_status = IncidentStatusUpdate::belonging_to(&all_incidents)
        .order(incident_status_update::dsl::created.desc())
        .inner_join(incident_status_type::table)
        .filter(incident_status_type::dsl::id.eq(incident_status_update::dsl::id))
        .load(&pool.get().unwrap())
        .unwrap()
        .grouped_by(&all_incidents);
    let incidents_and_status = all_incidents
        .into_iter()
        .zip(incident_days_status)
        .collect::<Vec<_>>();

    for n in 0..10 {
        let date = Utc::now().sub(Duration::days(n));

        let incident_on_day = incidents_and_status
            .iter()
            .filter(|i| i.0.created.date() == date.naive_utc().date())
            .cloned()
            .collect::<Vec<_>>();

        incident_days.push(IncidentDay {
            date: date.format("%Y-%m-%d").to_string(),
            incidents: incident_on_day,
        })
    }

    let template = IndexTemplate {
        projects: p,
        history_size,
        incident_days,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
        custom_html: settings.get_setting(CUSTOM_HTML),
    }
    .render()
    .expect("Unable to render template");

    HttpResponse::Ok().body(template)
}

//TODO: REST API
//TODO: SMS, webhooks, twitter, rss, atom
//TODO: admin ui
//TODO: multiple accounts
//TODO: SSO
//TODO: embed support

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let db = db::get_db_connection();
    let mailer = Arc::new(Mailer::default());

    if env::var("UPDATE").unwrap_or_else(|_| "1".to_string()) == "1" {
        spawn(run_update_job(mailer.clone(), db.clone()));
    }

    let port = env::var("HOST_PORT").unwrap_or_else(|_| "8102".to_string());
    let ip = env::var("HOST_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = format!("{}:{}", ip, port);

    tracing::info!("Running on http://{}", host);

    //TODO: store in config

    HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .data(PersistedSettings::new(db.clone()))
            .data(mailer.clone())
            .service(Files::new("/static", "./static"))
            .service(resource("/").to(root))
            .service(get_admin_login)
            .service(post_admin_login)
            .service(get_admin_dashboard)
            .service(post_admin_dashboard)
            .service(get_admin_subscriptions)
            .service(post_admin_subscriptions)
            .service(post_email_subscribe)
            .service(get_email_confirm)
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&settings::private_key())
                    .name("you-up-auth")
                    .secure(false),
            ))
    })
    .bind(host)?
    .run()
    .await
}

use crate::models::{Project, Status};
use actix_files::Files;
use actix_rt::spawn;

use actix_web::middleware::{Compress, Logger};
use actix_web::web::{resource, Data};
use actix_web::HttpServer;
use actix_web::{App, HttpResponse, Responder};
use askama::Template;
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenv::dotenv;

use crate::db::Database;
use crate::project_status::ProjectStatusTypes;
use crate::template_index::IndexTemplate;
use crate::template_tooltip::StatusTooltipTemplate;
use crate::update_job::run_update_job;
use chrono::{Timelike, Utc};
use diesel::query_dsl::methods::OrderDsl;
use std::cmp::max;
use std::convert::TryInto;
use std::env;
use std::ops::Sub;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod db;
pub mod models;
pub mod project_status;
pub mod schema;
pub mod template_index;
pub mod template_tooltip;
pub mod update_job;

#[derive(Clone)]
pub struct Downtime {
    pub duration: String,
}

#[derive(Clone)]
pub struct StatusDay<'a> {
    pub status: Vec<&'a Status>,
    pub date: String,
    pub downtime: Vec<Downtime>,
}

impl StatusDay<'_> {
    fn get_overall_status(&self) -> ProjectStatusTypes {
        let initial_status = if self.status.is_empty() {
            ProjectStatusTypes::Unknown
        } else if self.status.iter().all(|x| x.is_success()) {
            ProjectStatusTypes::Operational
        } else if self.status.iter().all(|x| !x.is_success()) {
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

    fn get_chart_status(&self) -> &[&Status] {
        &self.status[max(self.status.len() as i32 - 100, 0) as usize..self.status.len()]
    }
}

pub struct ProjectStatus<'a> {
    pub project: Project,
    pub days: Vec<StatusDay<'a>>,
    pub today: StatusDay<'a>,
}

async fn compute_downtime_periods(status_on_day: &[&Status]) -> Vec<Downtime> {
    let mut downtime = Vec::new();
    let mut downtime_period_start = None;

    for item in status_on_day.iter() {
        if item.is_success() {
            if let Some(tmp) = downtime_period_start {
                let period_duration = item.created.signed_duration_since(tmp);
                downtime.push(Downtime {
                    duration: format!(
                        "{} hours and {} minutes",
                        period_duration.num_hours(),
                        period_duration.num_minutes()
                    ),
                });
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
            .with_hour(23)
            .unwrap()
            .with_minute(59)
            .unwrap()
            .with_second(59)
            .unwrap();

        // If this day was in the past then take its end_of_day however if we are considering today then define the end to be the current time
        let clamped_end_of_day = end_of_day.min(Utc::now().naive_utc());

        let period_duration = clamped_end_of_day.signed_duration_since(tmp);
        if period_duration.num_minutes() > 0 {
            downtime.push(Downtime {
                duration: format!(
                    "{} hours and {} minutes",
                    period_duration.num_hours(),
                    period_duration.num_minutes()
                ),
            });
        }
    }

    downtime
}

pub async fn root(pool: Data<Database>) -> impl Responder {
    use self::schema::projects::dsl::*;
    use self::schema::status as stat;

    let projects_list = projects
        .load::<Project>(&pool.get().unwrap())
        .expect("Unable to load projects");
    let status_list = stat::dsl::status
        .order(stat::dsl::time.desc())
        .load::<Status>(&pool.get().unwrap())
        .expect("Unable to load status");

    let history_size = env::var("HISTORY_SIZE")
        .unwrap_or_else(|_| "30".into())
        .parse::<usize>()
        .unwrap_or(30);

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

            let status_on_day: Vec<&Status> = status_list
                .iter()
                .filter(|s| s.project == proj.id && s.created.date() == then.date())
                .collect();

            let downtime = compute_downtime_periods(&status_on_day).await;

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

    let template = IndexTemplate {
        projects: p,
        history_size,
    }
    .render()
    .expect("Unable to render template");

    HttpResponse::Ok().body(template)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db = db::get_db_connection();

    spawn(run_update_job(db.clone()));

    HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .service(Files::new("/static", "./static"))
            .service(resource("/").to(root))
            .wrap(Logger::default())
            .wrap(Compress::default())
    })
    .bind("0.0.0.0:8102")?
    .run()
    .await
}

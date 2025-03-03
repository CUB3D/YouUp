use crate::data::project_repository::ProjectRepositoryData;
use crate::data::status_repository::StatusRepositoryData;
use crate::models::Project;
use crate::project_status::ProjectStatusTypes;
use crate::settings::{CUSTOM_HTML, CUSTOM_SCRIPT, CUSTOM_STYLE, PersistedSettings};
use crate::template::template_admin_login::AdminLogin;
use crate::time_utils::get_days_from_month;
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder, get};
use askama::Template;
use chrono::{Datelike, Utc};
use std::ops::Sub;

pub struct Month {
    pub name: String,
    pub days: Vec<ProjectStatusTypes>,
    pub first_day_offset: u32,
}

#[derive(Template)]
#[template(path = "incidents.html")]
pub struct UptimeTemplate {
    pub projects: Vec<Project>,
    pub months: Vec<Month>,
    pub custom_script: String,
    pub custom_style: String,
    pub custom_html: String,
    pub admin_logged_in: bool,
}

#[get("/incidents")]
pub async fn get_incident_history(
    projects_repo: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    settings: Data<PersistedSettings>,
    identity: Option<Identity>,
) -> impl Responder {
    let projects = projects_repo.get_all_enabled_projects();
    let status_list = status_repo.get_status_last_90_days();

    let mut months = Vec::new();

    let month_range = 3;

    let now = Utc::now();
    for i in (0..month_range).rev() {
        let month = now.sub(chrono::Duration::weeks(i * 4));

        let status_on_month = status_list
            .iter()
            .filter(|s| s.created.month() == month.month() && s.created.year() == s.created.year())
            .collect::<Vec<_>>();

        let mut status_days = Vec::new();

        //TODO: length of month
        for day in 0..get_days_from_month(month.year(), month.month()) {
            let status_on_day = status_on_month
                .iter()
                .filter(|s| s.created.day0() == day)
                .collect::<Vec<_>>();

            status_days.push(if status_on_day.is_empty() {
                ProjectStatusTypes::Unknown
            } else if status_on_day.iter().all(|s| !s.is_success()) {
                ProjectStatusTypes::Failed
            } else if status_on_day.iter().all(|s| s.is_success()) {
                ProjectStatusTypes::Operational
            } else {
                ProjectStatusTypes::Failing
            })
        }

        //TODO: needs to find status day for each day in month then find the overall for each day and show that as a square

        let first_day_offset = month.with_day0(0).unwrap().weekday().num_days_from_monday();

        months.push(Month {
            name: month.format("%B").to_string(),
            days: status_days,
            first_day_offset,
        });
    }

    let body = UptimeTemplate {
        projects,
        months,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
        custom_html: settings.get_setting(CUSTOM_HTML),
        admin_logged_in: identity.is_logged_in(),
    }
    .render()
    .expect("Unable to render update template");

    HttpResponse::Ok().body(body)
}

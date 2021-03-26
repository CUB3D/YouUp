use crate::data::project_repository::ProjectRepositoryData;
use crate::data::status_repository::StatusRepositoryData;
use crate::settings;
use crate::template::index::downtime::compute_downtime_periods;
use crate::template::index::status_day::StatusDay;
use crate::template::index::template_index::ProjectStatus;
use actix_web::get;
use actix_web::web::Path;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use chrono::{Timelike, Utc};
use std::convert::TryInto;
use std::ops::Sub;

#[derive(Template)]
#[template(path = "embed.html")]
pub struct EmbedTemplate {
    pub proj_status: ProjectStatus,
    pub history_size: usize,
    pub custom_script: String,
    pub custom_style: String,
    pub custom_html: String,
}

#[get("/embed/{id}")]
pub async fn get_embed(
    id: Path<(i32,)>,
    projects: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
) -> impl Responder {
    if let Some(proj) = projects.get_project_by_id(id.0 .0) {
        let status_list: Vec<_> = status_repo.get_status_last_30_days();
        let history_size = settings::get_history_size();

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

        let ps = ProjectStatus {
            project: proj,
            days,
            today,
        };

        let template = EmbedTemplate {
            history_size,
            proj_status: ps,
            custom_script: "".to_string(),
            custom_style: "".to_string(),
            custom_html: "".to_string(),
        }
        .render()
        .expect("Unable to render template");

        HttpResponse::Ok().body(template)
    } else {
        HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/")
            .finish()
    }
}

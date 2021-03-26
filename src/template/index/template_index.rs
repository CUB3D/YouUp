use crate::data::project_repository::ProjectRepositoryData;
use crate::data::status_repository::StatusRepositoryData;
use crate::db::Database;
use crate::diesel::BelongingToDsl;
use crate::diesel::GroupedBy;
use crate::models::{IncidentStatusType, IncidentStatusUpdate, Incidents, Project, Status};
use crate::project_status::ProjectStatusTypes;
use crate::settings::{PersistedSettings, CUSTOM_HTML, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::index::downtime::{compute_downtime_periods, Downtime};
use crate::template::index::status_day::StatusDay;
use crate::template::template_admin_login::AdminLogin;
use crate::{settings, time_formatter};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use diesel::dsl::sql;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use std::convert::TryInto;
use std::ops::Sub;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub projects: Vec<ProjectStatus>,
    pub history_size: usize,
    pub incident_days: Vec<IncidentDay>,
    pub admin_logged_in: bool,
    pub custom_script: String,
    pub custom_style: String,
    pub custom_html: String,
}

impl IndexTemplate {
    pub fn is_operational_today(&self) -> bool {
        self.projects
            .iter()
            .all(|p| p.today.get_overall_status() == ProjectStatusTypes::Operational)
    }
}

pub struct IncidentDay {
    pub date: String,
    pub incidents: Vec<(Incidents, Vec<(IncidentStatusUpdate, IncidentStatusType)>)>,
}

pub struct ProjectStatus {
    pub project: Project,
    pub days: Vec<StatusDay>,
    pub today: StatusDay,
}

pub async fn root(
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    project_repo: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    identity: Identity,
) -> impl Responder {
    let projects_list = project_repo.get_all_projects();

    let status_list: Vec<_> = status_repo.get_status_last_30_days();

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

    use crate::schema::incident_status_type;
    use crate::schema::incident_status_update;
    use crate::schema::incidents::dsl::*;

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
        admin_logged_in: identity.is_logged_in(),
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
        custom_html: settings.get_setting(CUSTOM_HTML),
    }
    .render()
    .expect("Unable to render template");

    HttpResponse::Ok().body(template)
}

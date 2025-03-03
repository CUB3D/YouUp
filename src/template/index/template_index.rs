use crate::data::project_repository::ProjectRepositoryData;
use crate::data::status_repository::StatusRepositoryData;
use crate::db::Database;
use crate::diesel::BelongingToDsl;
use crate::diesel::GroupedBy;
use crate::models::{IncidentStatusType, IncidentStatusUpdate, Incidents, Project, Status};
use crate::project_status::ProjectStatusTypes;
use crate::settings::{CUSTOM_HTML, CUSTOM_SCRIPT, CUSTOM_STYLE, PersistedSettings};
use crate::template::index::downtime::Downtime;
use crate::template::index::status_day::StatusDay;
use crate::template::template_admin_login::AdminLogin;
use crate::{settings, time_formatter};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use diesel::dsl::sql;
use diesel::sql_types::Bool;
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

pub async fn compute_downtime_periods(status_on_day: &[Status]) -> Vec<Downtime> {
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

pub async fn root(
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    project_repo: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    identity: Option<Identity>,
) -> impl Responder {
    let projects_list = project_repo.get_all_enabled_projects();

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
        .filter(sql::<Bool>("created > DATE_SUB(NOW(), INTERVAL 10 day)"))
        .load::<Incidents>(&mut pool.get().unwrap())
        .unwrap();

    let incident_days_status = IncidentStatusUpdate::belonging_to(&all_incidents)
        .order(incident_status_update::dsl::created.desc())
        .inner_join(incident_status_type::table)
        .filter(incident_status_type::dsl::id.eq(incident_status_update::dsl::id))
        .load(&mut pool.get().unwrap())
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

#[cfg(test)]
mod test {
    use crate::models::Status;
    use crate::template::index::template_index::compute_downtime_periods;
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
                created: Utc
                    .with_ymd_and_hms(2020, 9, 25, 23, 0, 0)
                    .unwrap()
                    .naive_utc(),
                // Utc::now().naive_utc().sub(chrono::Duration::hours(1)),
                status_code: 200,
                id: 2,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc
                    .with_ymd_and_hms(2020, 9, 25, 1, 0, 0)
                    .unwrap()
                    .naive_utc(),
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
                created: Utc
                    .with_ymd_and_hms(2020, 9, 25, 0, 0, 0)
                    .unwrap()
                    .naive_utc(),
                status_code: 404,
                id: 1,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc
                    .with_ymd_and_hms(2020, 9, 25, 12, 0, 0)
                    .unwrap()
                    .naive_utc(),
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

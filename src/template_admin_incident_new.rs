use crate::data::incident_repository::IncidentRepositoryData;
use crate::data::project_repository::ProjectRepositoryData;
use crate::db::Database;
use crate::models::{IncidentStatusType, NewIncident, Project};
use crate::schema::incident_status_type::dsl::incident_status_type;
use crate::schema::projects::dsl::projects;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use diesel::RunQueryDsl;
use serde::Deserialize;
use tracing_futures::Instrument;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin_incidents_new.html")]
pub struct AdminNewIncidentTemplate {
    pub status_types: Vec<IncidentStatusType>,
    pub projects: Vec<Project>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize)]
pub struct ProjectUpdate {
    pub project: String,
    pub status_type: String,
    pub date: String,
    pub message: String,
}

async fn admin_incidents_new(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin")
            .finish();
    }

    let status_types = incident_status_type
        .load::<IncidentStatusType>(&pool.get().unwrap())
        .expect("Unable to load incident status types");

    // let projects_list = project_repo.get_all_projects();

    let projects_list = projects
        .load::<Project>(&pool.get().unwrap())
        .expect("Unable to load projects");

    let template = AdminNewIncidentTemplate {
        status_types,
        projects: projects_list,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/incidents/new")]
pub async fn get_admin_incidents_new(
    id: Identity,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin Incidents New GET", request_id = %request_id);

    admin_incidents_new(id, pool, settings)
        .instrument(span)
        .await
}

#[post("/admin/incidents/new")]
pub async fn post_admin_incidents_new(
    id: Identity,
    pool: Data<Database>,
    project_repo: ProjectRepositoryData,
    incident_repo: IncidentRepositoryData,
    settings: Data<PersistedSettings>,
    form_data: Form<ProjectUpdate>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin Incidents New POST", request_id = %request_id);

    let project_id = project_repo
        .get_project_by_name(&form_data.project)
        .first()
        .map(|f| f.id)
        .expect("Unable to find project with given id");

    incident_repo.add_incident(NewIncident {
        project: project_id,
    });

    admin_incidents_new(id, pool, settings)
        .instrument(span)
        .await
}

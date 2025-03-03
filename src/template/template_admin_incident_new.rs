use crate::data::incident_repository::IncidentRepositoryData;
use crate::data::project_repository::ProjectRepositoryData;
use crate::db::Database;
use crate::models::{IncidentStatusType, NewIncident, NewIncidentStatusUpdate, Project};
use crate::schema::incident_status_type::dsl::incident_status_type;
use crate::schema::projects::dsl::projects;
use crate::settings::{CUSTOM_SCRIPT, CUSTOM_STYLE, PersistedSettings};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder, web::Form};
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
    id: Option<Identity>,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let status_types = incident_status_type
        .load::<IncidentStatusType>(&mut pool.get().unwrap())
        .expect("Unable to load incident status types");

    // let projects_list = project_repo.get_all_projects();

    let projects_list = projects
        .load::<Project>(&mut pool.get().unwrap())
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
    id: Option<Identity>,
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
    id: Option<Identity>,
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

    let incident = incident_repo
        .get_all_incidents()
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0);
    tracing::debug!("Adding status update to incident {}", incident);

    let status_type = incident_repo
        .get_incident_status_type_by_title(&form_data.status_type)
        .expect("Unknown status type");
    tracing::debug!(
        "Using status type mapping {} -> {:?}",
        form_data.status_type,
        status_type
    );

    incident_repo.add_status_update(NewIncidentStatusUpdate {
        incident,
        message: form_data.message.clone(),
        status_type: status_type.id,
    });

    admin_incidents_new(id, pool, settings)
        .instrument(span)
        .await
}

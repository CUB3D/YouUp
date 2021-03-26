use crate::data::project_repository::ProjectRepositoryData;
use crate::data::status_repository::StatusRepositoryData;
use crate::db::Database;
use crate::models::Project;
use crate::schema::projects::dsl::projects;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::{Data, Path};
use actix_web::{HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "admin_project_details.html")]
pub struct ProjectDetailsTemplate {
    pub project: Project,
    pub custom_script: String,
    pub custom_style: String,
    pub uptime_percentage: String,
}

#[derive(Deserialize)]
pub struct ProjectUpdate {}

async fn admin_project_details(
    id: Identity,
    path: Path<(i32,)>,
    project: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin")
            .finish();
    }

    let project = project.get_project_by_id(path.0 .0);
    if project.is_none() {
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin/dashboard")
            .finish();
    }
    let project = project.unwrap();

    let period_status = status_repo.get_status_last_30_days();
    let status = period_status
        .iter()
        .filter(|s| s.project == project.id)
        .collect::<Vec<_>>();
    let uptime_percent =
        ((status.iter().filter(|s| s.is_success()).count() as f32) / (status.len() as f32)) * 100.0;

    let template = ProjectDetailsTemplate {
        project,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
        uptime_percentage: format!("{:3.2}", uptime_percent),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/project/{id}")]
pub async fn get_admin_project_details(
    id: Identity,
    path: Path<(i32,)>,
    project: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    admin_project_details(id, path, project, status_repo, settings).await
}

#[post("/admin/projects/{id}")]
pub async fn post_admin_project_details(
    id: Identity,
    path: Path<(i32,)>,
    project: ProjectRepositoryData,
    status_repo: StatusRepositoryData,
    settings: Data<PersistedSettings>,
    // _updates: Option<Form<ProjectUpdate>>,
) -> impl Responder {
    admin_project_details(id, path, project, status_repo, settings).await
}

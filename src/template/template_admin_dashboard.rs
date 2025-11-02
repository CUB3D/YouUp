use crate::data::project_repository::ProjectRepositoryData;
use crate::db::Database;
use crate::models::Project;
use crate::schema::projects::dsl::projects;
use crate::settings::{CUSTOM_SCRIPT, CUSTOM_STYLE, PersistedSettings};
use crate::template::template_admin_login::AdminLogin;
use crate::{get_db, get_pool};
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{HttpResponse, web::Form};
use askama::Template;
use diesel::RunQueryDsl;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
pub struct AdminDashboardTemplate {
    pub projects: Vec<Project>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize, Debug)]
pub struct ProjectUpdate {
    project_id: i32,
    name: String,
    description: String,
    url: String,
    enabled: Option<String>,
}

async fn admin_dashboard(pool: Database, settings: Data<PersistedSettings>) -> HttpResponse {
    let mut pool = get_pool!(pool);

    let projects_list = projects
        .load::<Project>(&mut pool)
        .expect("Unable to load projects");

    let template = AdminDashboardTemplate {
        projects: projects_list,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/dashboard")]
pub async fn get_admin_dashboard(
    id: Option<Identity>,
    settings: Data<PersistedSettings>,
) -> HttpResponse {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let pool = get_db!();

    admin_dashboard(pool, settings).await
}

#[post("/admin/dashboard")]
pub async fn post_admin_dashboard(
    id: Option<Identity>,
    settings: Data<PersistedSettings>,
    updates: Form<ProjectUpdate>,
    project: ProjectRepositoryData,
) -> HttpResponse {
    let _span = tracing::info_span!("Admin Project Update", ?updates);

    let pool = get_db!();

    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let _ = project.update_project(
        updates.project_id,
        updates.name.as_str(),
        updates.enabled.clone().unwrap_or_default() == "on",
        &updates.url,
        &updates.description,
    );

    admin_dashboard(pool, settings).await
}

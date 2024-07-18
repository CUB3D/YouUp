use crate::db::Database;
use crate::models::Project;
use crate::schema::projects::dsl::projects;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{web::Form, HttpResponse, Responder};
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

#[derive(Deserialize)]
pub struct ProjectUpdate {}

async fn admin_dashboard(
    id: Option<Identity>,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let projects_list = projects
        .load::<Project>(&mut pool.get().unwrap())
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
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    admin_dashboard(id, pool, settings).await
}

#[post("/admin/dashboard")]
pub async fn post_admin_dashboard(
    id: Option<Identity>,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
    _updates: Option<Form<ProjectUpdate>>,
) -> impl Responder {
    admin_dashboard(id, pool, settings).await
}

use crate::db::Database;
use crate::models::Project;
use crate::schema::projects::dsl::projects;
use crate::template_admin_login::AdminLogin;
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
}

#[get("/admin/dashboard")]
pub async fn get_admin_dashboard(id: Identity, pool: Data<Database>) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin")
            .finish();
    }

    let projects_list = projects
        .load::<Project>(&pool.get().unwrap())
        .expect("Unable to load projects");

    let template = AdminDashboardTemplate {
        projects: projects_list,
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

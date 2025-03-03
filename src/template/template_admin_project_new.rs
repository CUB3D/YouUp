use crate::data::project_repository::ProjectRepositoryData;
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::web::Query;
use actix_web::{HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProjectNew {
    pub _new_name: String,
}

async fn admin_project_new(
    id: Option<Identity>,
    projects: ProjectRepositoryData,
    new: ProjectNew,
) -> impl Responder {
    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let _ = projects.create(&new._new_name);

    HttpResponse::TemporaryRedirect()
        .append_header((http::header::LOCATION.as_str(), "/admin/dashboard"))
        .finish()
}

#[get("/admin/project/new")]
pub async fn get_admin_project_new(
    id: Option<Identity>,
    new: Query<ProjectNew>,
    projects: ProjectRepositoryData,
) -> impl Responder {
    let _span = tracing::info_span!("Admin Project New");

    admin_project_new(id, projects, new.0).await
}

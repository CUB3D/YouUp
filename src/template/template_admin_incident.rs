use crate::data::incident_repository::IncidentRepositoryData;
use crate::models::Incidents;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin_incidents.html")]
pub struct AdminSubscriptionTemplate {
    pub incidents: Vec<Incidents>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize)]
pub struct ProjectUpdate {}

async fn admin_incidents(
    id: Identity,
    settings: Data<PersistedSettings>,
    incident_repo: IncidentRepositoryData,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin Incidents", request_id = %request_id);
    let _guard = span.enter();

    if !id.is_logged_in() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin"))
            .finish();
    }

    let incidents = incident_repo.get_all_incidents();

    let template = AdminSubscriptionTemplate {
        incidents,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/incidents")]
pub async fn get_admin_incidents(
    id: Identity,
    settings: Data<PersistedSettings>,
    incident_repo: IncidentRepositoryData,
) -> impl Responder {
    admin_incidents(id, settings, incident_repo).await
}

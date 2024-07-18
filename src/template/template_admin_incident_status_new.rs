use crate::data::incident_repository::IncidentRepositoryData;
use crate::db::Database;
use crate::diesel::RunQueryDsl;
use crate::models::{IncidentStatusType, NewIncidentStatusUpdate};
use crate::schema::incident_status_type::dsl::incident_status_type;
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use crate::template::template_admin_login::AdminLogin;
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::{Data, Path};
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;
use tracing_futures::Instrument;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin_incident_status_new.html")]
pub struct AdminNewIncidentStatusTemplate {
    pub incident_id: i32,
    pub status_types: Vec<IncidentStatusType>,
    pub custom_script: String,
    pub custom_style: String,
}

#[derive(Deserialize)]
pub struct StatusUpdate {
    pub status_type: String,
    pub date: String,
    pub message: String,
}

async fn admin_incident_status_new(
    incident_id: i32,
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

    let template = AdminNewIncidentStatusTemplate {
        incident_id,
        status_types,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[get("/admin/incident/{id}/status/new")]
pub async fn get_admin_incident_status_new(
    path: Path<(i32,)>,
    id: Option<Identity>,
    pool: Data<Database>,
    settings: Data<PersistedSettings>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin Incidents New Status GET", request_id = %request_id);

    admin_incident_status_new(path.into_inner().0, id, pool, settings)
        .instrument(span)
        .await
}

#[post("/admin/incident/{id}/status/new")]
pub async fn post_admin_incident_status_new(
    path: Path<(i32,)>,
    id: Option<Identity>,
    pool: Data<Database>,
    incident_repo: IncidentRepositoryData,
    settings: Data<PersistedSettings>,
    form_data: Form<StatusUpdate>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin Incidents New Status POST", request_id = %request_id);

    let status_type = incident_repo
        .get_incident_status_type_by_title(&form_data.status_type)
        .expect("Unknown status type");
    tracing::debug!(
        "Using status type mapping {} -> {:?}",
        form_data.status_type,
        status_type
    );

    let (incident,) = path.into_inner();

    incident_repo.add_status_update(NewIncidentStatusUpdate {
        incident,
        message: form_data.message.clone(),
        status_type: status_type.id,
    });

    admin_incident_status_new(incident, id, pool, settings)
        .instrument(span)
        .await
}

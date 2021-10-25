use crate::data::incident_repository::IncidentRepositoryData;
use crate::data::project_repository::ProjectRepositoryData;
use crate::models::{IncidentStatusType, IncidentStatusUpdate, Incidents, Project};
use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use actix_web::get;
use actix_web::web::{Data, Path};
use actix_web::{HttpResponse, Responder};
use askama::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "incident_details.html")]
pub struct IncidentDetailsTemplate {
    pub incident: Incidents,
    pub status_updates: Vec<(IncidentStatusUpdate, IncidentStatusType)>,
    pub project: Project,
    pub custom_script: String,
    pub custom_style: String,
}

#[get("/incident/{id}")]
pub async fn get_incident_details(
    id: Path<(i32,)>,
    settings: Data<PersistedSettings>,
    incidents: IncidentRepositoryData,
    projects: ProjectRepositoryData,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span =
        tracing::info_span!("Incident", request_id = %request_id, incident_id = id.as_ref().0);
    let _guard = span.enter();

    let incident = incidents.get_incident_by_id(id.as_ref().0);
    //TODO: can we do this with a join?
    let project = projects.get_project_by_id(incident.project);

    if project.is_none() {
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION, "/"))
            .finish();
    }

    let project = project.unwrap();

    let status_updates = incidents.get_status_updates_by_incident(&incident);
    tracing::debug!(
        "Got {} status updates for incident {}",
        status_updates.len(),
        id.as_ref().0
    );

    let template = IncidentDetailsTemplate {
        incident,
        status_updates,
        project,
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
    .render()
    .expect("Unable to render template");

    HttpResponse::Ok().body(template)
}

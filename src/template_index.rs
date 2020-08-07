use crate::{IncidentDay, ProjectStatus};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub projects: Vec<ProjectStatus>,
    pub history_size: usize,
    pub incident_days: Vec<IncidentDay>,
}

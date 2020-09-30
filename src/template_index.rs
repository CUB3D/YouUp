use crate::project_status::ProjectStatusTypes;
use crate::{IncidentDay, ProjectStatus};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub projects: Vec<ProjectStatus>,
    pub history_size: usize,
    pub incident_days: Vec<IncidentDay>,
    pub custom_script: String,
    pub custom_style: String,
    pub custom_html: String,
}

impl IndexTemplate {
    pub fn is_operational_today(&self) -> bool {
        self.projects
            .iter()
            .all(|p| p.today.get_overall_status() == ProjectStatusTypes::Operational)
    }
}

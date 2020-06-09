use crate::ProjectStatus;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub projects: Vec<ProjectStatus<'a>>,
    pub history_size: usize,
}

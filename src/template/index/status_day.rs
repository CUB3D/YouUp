use crate::models::Status;
use crate::project_status::ProjectStatusTypes;
use crate::template::index::downtime::Downtime;
use crate::template::template_tooltip::StatusTooltipTemplate;
use askama::Template;

#[derive(Clone)]
pub struct StatusDay {
    pub status: Vec<Status>,
    pub date: String,
    pub downtime: Vec<Downtime>,
}

impl StatusDay {
    pub fn get_overall_status(&self) -> ProjectStatusTypes {
        if self.status.is_empty() {
            return ProjectStatusTypes::Unknown;
        }
        if self.downtime.is_empty() && !self.status.is_empty() {
            return ProjectStatusTypes::Operational;
        }

        let initial_status = if self.status.iter().all(|x| !x.is_success()) {
            ProjectStatusTypes::Failed
        } else {
            //TODO: apply more logic here, should really only be failed if more fail than success (based on amount of time)
            ProjectStatusTypes::Failing
        };

        // If a system was failing and is now working again show it as recovering to indicate this
        if initial_status == ProjectStatusTypes::Failing
            && self.status.last().map(|s| s.is_success()).unwrap_or(false)
        {
            ProjectStatusTypes::Recovering
        } else {
            initial_status
        }
    }

    pub fn get_tooltip(&self) -> String {
        StatusTooltipTemplate { day: self.clone() }
            .render()
            .expect("Unable to render tooltip")
    }

    pub fn avg_request_time(&self) -> u32 {
        if self.status.is_empty() {
            0
        } else {
            self.status.iter().map(|s| s.time).sum::<i32>() as u32 / (self.status.len() as u32)
        }
    }

    pub fn get_chart_status(&self) -> &[Status] {
        self.status.as_slice() //[max(self.status.len() - 100, 0usize)..self.status.len()]
    }
}

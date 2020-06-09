use crate::StatusDay;
use askama::Template;

#[derive(Template)]
#[template(path = "status_tooltip.html")]
pub struct StatusTooltipTemplate<'a> {
    pub day: StatusDay<'a>,
}

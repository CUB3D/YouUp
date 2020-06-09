#[derive(Template)]
#[template(path = "status_tooltip.html")]
pub struct StatusTooltipTemplate<'a> {
    day: StatusDay<'a>,
}
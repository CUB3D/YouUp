use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq)]
pub enum ProjectStatusTypes {
    Operational,
    Recovering,
    Failing,
    Failed,
    Unknown,
}

impl ProjectStatusTypes {
    pub fn get_colour(&self) -> String {
        match self {
            ProjectStatusTypes::Operational => "#00FF00".into(),
            ProjectStatusTypes::Recovering => "#FFFF00".into(),
            ProjectStatusTypes::Failing => "#FFAA00".into(),
            ProjectStatusTypes::Failed => "#FF0000".into(),
            ProjectStatusTypes::Unknown => "#808080".into(),
        }
    }
}

impl Display for ProjectStatusTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ProjectStatusTypes::Operational => "Operational",
            ProjectStatusTypes::Recovering => "Recovering",
            ProjectStatusTypes::Failing => "Failing",
            ProjectStatusTypes::Failed => "Failed",
            ProjectStatusTypes::Unknown => "Unknown",
        })
    }
}

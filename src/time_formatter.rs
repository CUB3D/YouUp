use chrono::Duration;
use std::cmp::max;

pub fn format_duration(d: &Duration) -> String {
    let hours = d.num_hours();
    let minutes = max(0, d.num_minutes() - hours * 60);

    let mut output = String::new();
    if hours > 0 {
        if hours == 1 {
            output += "1 hour ";
        } else {
            output += format!("{} hours ", hours).as_str();
        }
    }
    if minutes > 0 {
        output += format!("{} minutes", minutes).as_str();
    }

    output.trim().to_string()
}

#[cfg(test)]
pub mod test {
    use crate::time_formatter::format_duration;
    use chrono::Duration;

    #[test]
    fn basic_format() {
        assert_eq!(format_duration(&Duration::minutes(120)), "2 hours");
        assert_eq!(
            format_duration(&Duration::minutes(470)),
            "7 hours 50 minutes"
        );
        assert_eq!(
            format_duration(&Duration::minutes(109)),
            "1 hour 49 minutes"
        );
    }
}

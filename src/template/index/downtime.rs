use crate::{time_formatter, settings};
use chrono::{Utc, Duration, NaiveDateTime, Timelike};
use crate::models::Status;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Downtime {
    pub duration: String,
}

pub async fn compute_downtime_periods(status_on_day: &[Status]) -> Vec<Downtime> {
    if !status_on_day.is_empty() && status_on_day.iter().all(|s| !s.is_success()) {
        let first_day = status_on_day.first().unwrap().created;

        // Get 00:00:00 on the next day
        let end_of_first_day = first_day
            .checked_add_signed(Duration::days(1))
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        // Find the duration between the last point that we have info about and the first point
        // If the end of the day is in the future, then this downtime occurs today, and we will state that it is down until now
        // If this day is in the past then we will assume it was down for the rest of the day
        //TODO: we might want to check when it's up the next day
        // also might want to look back to before the first result of the day?
        let duration = Utc::now()
            .naive_utc()
            .min(end_of_first_day)
            .signed_duration_since(first_day);

        return vec![Downtime {
            duration: time_formatter::format_duration(&duration),
        }];
    }

    let mut downtime = Vec::new();
    let mut downtime_period_start: Option<NaiveDateTime> = status_on_day
        .first()
        .map(|e| Some(e.created))
        .unwrap_or(None);

    for item in status_on_day.iter() {
        if item.is_success() {
            if let Some(tmp) = downtime_period_start {
                let period_duration = tmp.signed_duration_since(item.created);

                if period_duration.num_minutes() > settings::get_minimum_downtime_minutes() {
                    downtime.push(Downtime {
                        duration: time_formatter::format_duration(&period_duration),
                    });
                }
                downtime_period_start = None;
            }
        } else {
            // If we are currently down then we will skip until we find the next up
            if downtime_period_start.is_none() {
                downtime_period_start = Some(item.created)
            }
        }
    }
    if let Some(tmp) = downtime_period_start {
        // If it was still down at the last reading of the day then assume it was down for all of the rest of that day

        let time_of_first_request = status_on_day.first().map(|s| s.created).unwrap();

        let end_of_day = time_of_first_request
            .checked_add_signed(Duration::days(1))
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        // If this day was in the past then take its end_of_day however if we are considering today then define the end to be the current time
        let clamped_end_of_day = end_of_day.min(Utc::now().naive_utc());

        let period_duration = clamped_end_of_day.signed_duration_since(tmp);
        if period_duration.num_minutes() > settings::get_minimum_downtime_minutes() {
            downtime.push(Downtime {
                duration: time_formatter::format_duration(&period_duration),
            });
        }
    }

    downtime
}

#[cfg(test)]
mod test {
    use crate::models::Status;
    use chrono::{TimeZone, Utc};
    use std::ops::Sub;
    use crate::template::index::downtime::compute_downtime_periods;

    #[actix_rt::test]
    async fn compute_simple_downtime() {
        let x = compute_downtime_periods(&[Status {
            created: Utc::now().naive_utc(),
            status_code: 200,
            id: 0,
            project: 0,
            time: 0,
        }])
            .await;

        assert!(x.is_empty())
    }

    #[actix_rt::test]
    async fn compute_downtime() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc::now().naive_utc(),
                status_code: 200,
                id: 3,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc::now().naive_utc().sub(chrono::Duration::hours(1)),
                status_code: 503,
                id: 2,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc::now().naive_utc().sub(chrono::Duration::hours(2)),
                status_code: 200,
                id: 1,
                project: 0,
                time: 10,
            },
        ])
            .await;

        assert_eq!(x.first().unwrap().duration, "59 minutes");
        assert_eq!(x.len(), 1);
    }

    #[actix_rt::test]
    async fn compute_downtime_end_of_day() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(23, 0, 0).naive_utc(),
                // Utc::now().naive_utc().sub(chrono::Duration::hours(1)),
                status_code: 200,
                id: 2,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(1, 0, 0).naive_utc(),
                status_code: 404,
                id: 1,
                project: 0,
                time: 10,
            },
        ])
            .await;

        assert_eq!(x.first().unwrap().duration, "23 hours");
        assert_eq!(x.len(), 1);
    }

    #[actix_rt::test]
    async fn compute_downtime_never_up() {
        let x = compute_downtime_periods(&[
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(0, 0, 0).naive_utc(),
                status_code: 404,
                id: 1,
                project: 0,
                time: 10,
            },
            Status {
                created: Utc.ymd(2020, 9, 25).and_hms(12, 0, 0).naive_utc(),
                status_code: 404,
                id: 2,
                project: 0,
                time: 10,
            },
        ])
            .await;

        assert_eq!(x.first().unwrap().duration, "24 hours");
        assert_eq!(x.len(), 1);
    }
}

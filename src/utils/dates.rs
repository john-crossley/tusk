use chrono::{Duration, NaiveDate};

pub fn todays_date() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn parse_ymd(d: &str) -> Result<NaiveDate, String> {
    match d {
        "yesterday" => Ok(todays_date() - Duration::days(1)),
        "tomorrow" => Ok(todays_date() + Duration::days(1)),
        _ => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| format!("Invalid date '{d}'. Use YYYY-MM-DD, e.g. 2025-09-14")),
    }
}
use chrono::{Duration, NaiveDate};

pub fn today_date() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn parse_ymd(d: &str) -> Result<NaiveDate, String> {
    match d {
        "yesterday" => Ok(today_date() - Duration::days(1)),
        "tomorrow" => Ok(today_date() + Duration::days(1)),
        _ => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| format!("Invalid date '{d}'. Use YYYY-MM-DD, e.g. 2025-09-14")),
    }
}
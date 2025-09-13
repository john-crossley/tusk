use chrono::NaiveDate;

pub fn today_date() -> NaiveDate {
    chrono::Local::now().date_naive()
}

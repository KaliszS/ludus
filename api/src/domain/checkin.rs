use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Checkin {
    pub day: NaiveDate,
    pub times: i32,
}

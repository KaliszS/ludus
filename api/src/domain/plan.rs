use chrono::{Datelike, Days, Months, NaiveDate};
use uuid::Uuid;

use super::habit::Habit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Period {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "day" => Some(Self::Day),
            "week" => Some(Self::Week),
            "month" => Some(Self::Month),
            "quarter" => Some(Self::Quarter),
            "year" => Some(Self::Year),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
            Self::Year => "year",
        }
    }

    /// Start of the window `n` periods before the one containing `on`.
    pub fn step_back(self, on: NaiveDate, n: u32) -> NaiveDate {
        let (start, _) = self.window(on);
        match self {
            Self::Day => start - Days::new(u64::from(n)),
            Self::Week => start - Days::new(u64::from(n) * 7),
            Self::Month => start - Months::new(n),
            Self::Quarter => start - Months::new(n * 3),
            Self::Year => start - Months::new(n * 12),
        }
    }

    /// Half-open [start, end) containing `on`. Weeks start on Monday.
    pub fn window(self, on: NaiveDate) -> (NaiveDate, NaiveDate) {
        let start = match self {
            Self::Day => on,
            Self::Week => on - Days::new(u64::from(on.weekday().num_days_from_monday())),
            Self::Month => on.with_day(1).expect("day 1 always valid"),
            Self::Quarter => {
                let month = (on.month() - 1) / 3 * 3 + 1;
                NaiveDate::from_ymd_opt(on.year(), month, 1).expect("quarter start always valid")
            }
            Self::Year => NaiveDate::from_ymd_opt(on.year(), 1, 1).expect("Jan 1 always valid"),
        };
        let end = match self {
            Self::Day => start + Days::new(1),
            Self::Week => start + Days::new(7),
            Self::Month => start + Months::new(1),
            Self::Quarter => start + Months::new(3),
            Self::Year => start + Months::new(12),
        };
        (start, end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// Sum of the recorded amounts.
    Amount,
    /// Number of days with any check-in, so amounts do not distort a mixed group.
    Occurrences,
}

impl Measure {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "amount" => Some(Self::Amount),
            "occurrences" => Some(Self::Occurrences),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Amount => "amount",
            Self::Occurrences => "occurrences",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlanLevel {
    pub id: Uuid,
    pub name: String,
    pub period: Period,
    pub position: f64,
}

/// One member is an ordinary quota; several express "any N from this set".
#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: Uuid,
    pub level_id: Uuid,
    /// Set by the user when member names are too long to read, e.g. "Cardio".
    pub name: Option<String>,
    pub quota: f64,
    pub measure: Measure,
    pub position: f64,
    pub habit_ids: Vec<Uuid>,
}

/// One past period reduced to how many of each level's quotas were reached.
#[derive(Debug, Clone)]
pub struct PeriodOutcome {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub levels: Vec<LevelOutcome>,
}

#[derive(Debug, Clone)]
pub struct LevelOutcome {
    pub level: PlanLevel,
    pub reached: usize,
    pub total: usize,
}

impl LevelOutcome {
    pub fn met(&self) -> bool {
        self.total > 0 && self.reached == self.total
    }
}

#[derive(Debug, Clone)]
pub struct PlanProgress {
    pub period: Period,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub levels: Vec<LevelProgress>,
}

#[derive(Debug, Clone)]
pub struct LevelProgress {
    pub level: PlanLevel,
    pub items: Vec<RequirementProgress>,
    /// Consecutive periods completed, counting back from the current one. An
    /// unfinished current period does not break it.
    pub streak: u32,
}

impl LevelProgress {
    pub fn met(&self) -> bool {
        !self.items.is_empty() && self.items.iter().all(RequirementProgress::met)
    }
}

#[derive(Debug, Clone)]
pub struct RequirementProgress {
    pub id: Uuid,
    pub name: Option<String>,
    pub habits: Vec<Habit>,
    pub quota: f64,
    pub measure: Measure,
    pub done: f64,
}

impl RequirementProgress {
    pub fn met(&self) -> bool {
        self.done >= self.quota
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn step_back_walks_whole_periods() {
        assert_eq!(Period::Week.step_back(d("2026-09-22"), 2), d("2026-09-07"));
        assert_eq!(Period::Month.step_back(d("2026-09-22"), 3), d("2026-06-01"));
        assert_eq!(Period::Day.step_back(d("2026-09-22"), 1), d("2026-09-21"));
        assert_eq!(Period::Year.step_back(d("2026-09-22"), 1), d("2025-01-01"));
    }

    #[test]
    fn windows_are_half_open() {
        assert_eq!(
            Period::Week.window(d("2026-09-16")),
            (d("2026-09-14"), d("2026-09-21"))
        );
        assert_eq!(
            Period::Month.window(d("2026-09-16")),
            (d("2026-09-01"), d("2026-10-01"))
        );
        assert_eq!(
            Period::Quarter.window(d("2026-09-16")),
            (d("2026-07-01"), d("2026-10-01"))
        );
        assert_eq!(
            Period::Year.window(d("2026-09-16")),
            (d("2026-01-01"), d("2027-01-01"))
        );
        assert_eq!(
            Period::Day.window(d("2026-09-16")),
            (d("2026-09-16"), d("2026-09-17"))
        );
    }

    #[test]
    fn week_window_holds_on_monday_and_sunday() {
        let expected = (d("2026-09-14"), d("2026-09-21"));
        assert_eq!(Period::Week.window(d("2026-09-14")), expected);
        assert_eq!(Period::Week.window(d("2026-09-20")), expected);
    }
}

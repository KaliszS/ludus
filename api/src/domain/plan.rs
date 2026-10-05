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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Medal {
    Bronze,
    Silver,
    Gold,
}

impl Medal {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bronze" => Some(Self::Bronze),
            "silver" => Some(Self::Silver),
            "gold" => Some(Self::Gold),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bronze => "bronze",
            Self::Silver => "silver",
            Self::Gold => "gold",
        }
    }
}

/// A level a plan can be reached at. The name and the medal are both optional:
/// a tier can be just "the second one", and a plan with one tier is a plain plan.
#[derive(Debug, Clone)]
pub struct Tier {
    pub id: Uuid,
    pub name: Option<String>,
    pub medal: Option<Medal>,
    pub position: f64,
    /// Retired tiers stay because past versions still give quotas for them.
    pub retired_on: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub period: Period,
    pub position: f64,
    /// The day it was archived. It no longer counts from the period holding that day.
    pub archived_on: Option<NaiveDate>,
    /// Lowest first, retired ones included.
    pub tiers: Vec<Tier>,
}

impl Plan {
    /// Whether the plan was still in use when the period ending at `end` closed.
    pub fn counts_before(&self, end: NaiveDate) -> bool {
        self.archived_on.is_none_or(|day| day >= end)
    }
}

/// One member is an ordinary quota; several express "any N from this set".
///
/// A row is one version of a requirement, in force from `valid_from` up to
/// `valid_to`, so a period is judged by what was asked of it at the time.
#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: Uuid,
    pub plan_id: Uuid,
    /// Set by the user when member names are too long to read, e.g. "Cardio".
    pub name: Option<String>,
    /// One per tier this goal takes part in, in tier order; a tier left out is skipped.
    pub quotas: Vec<TierQuota>,
    pub measure: Measure,
    pub position: f64,
    pub habit_ids: Vec<Uuid>,
    pub valid_from: NaiveDate,
    /// Exclusive. `None` is the version in force now.
    pub valid_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TierQuota {
    pub tier_id: Uuid,
    pub quota: f64,
}

impl Requirement {
    /// Both bounds sit on period starts, so the period's first day decides.
    pub fn in_force_at(&self, start: NaiveDate) -> bool {
        self.valid_from <= start && self.valid_to.is_none_or(|to| to > start)
    }

    pub fn quota_at(&self, tier_id: Uuid) -> Option<f64> {
        self.quotas
            .iter()
            .find(|quota| quota.tier_id == tier_id)
            .map(|quota| quota.quota)
    }
}

/// One tier as a period left it.
#[derive(Debug, Clone)]
pub struct TierStanding {
    pub tier: Tier,
    pub reached: bool,
}

/// Climbs a plan's tiers for one period, lowest first. Only tiers some requirement
/// asks at take part, and a tier counts as reached only together with every tier
/// below it: clearing the top while missing the middle earns the bottom alone.
pub fn standing(
    tiers: &[Tier],
    asked: &[Requirement],
    done: impl Fn(&Requirement) -> f64,
) -> Vec<TierStanding> {
    let mut climbing = true;
    tiers
        .iter()
        .filter(|tier| {
            asked
                .iter()
                .any(|requirement| requirement.quota_at(tier.id).is_some())
        })
        .map(|tier| {
            climbing = climbing
                && asked.iter().all(|requirement| {
                    requirement
                        .quota_at(tier.id)
                        .is_none_or(|quota| done(requirement) >= quota)
                });
            TierStanding {
                tier: tier.clone(),
                reached: climbing,
            }
        })
        .collect()
}

/// A period counts toward the streak once its lowest asking tier is reached;
/// everything above that is extra.
pub fn met(standing: &[TierStanding]) -> bool {
    standing.first().is_some_and(|tier| tier.reached)
}

pub fn medals(standing: &[TierStanding]) -> Vec<Medal> {
    standing
        .iter()
        .filter(|tier| tier.reached)
        .filter_map(|tier| tier.tier.medal)
        .collect()
}

/// One stretch of a plan's life during which the same requirements were in force.
#[derive(Debug, Clone)]
pub struct PlanEra {
    pub from: NaiveDate,
    /// Exclusive. `None` is the stretch in force now.
    pub to: Option<NaiveDate>,
    pub requirements: Vec<Requirement>,
}

/// Cuts a plan's versions into the stretches between changes, newest first. Every
/// date a version starts or ends is a boundary, so each stretch shows exactly what
/// the plan asked then - including an empty one after everything was removed.
pub fn eras(versions: &[Requirement]) -> Vec<PlanEra> {
    let mut bounds: Vec<NaiveDate> = versions
        .iter()
        .flat_map(|version| std::iter::once(version.valid_from).chain(version.valid_to))
        .collect();
    bounds.sort_unstable();
    bounds.dedup();

    let mut eras: Vec<PlanEra> = bounds
        .iter()
        .enumerate()
        .map(|(index, from)| {
            let mut requirements: Vec<Requirement> = versions
                .iter()
                .filter(|version| version.in_force_at(*from))
                .cloned()
                .collect();
            requirements.sort_by(|a, b| a.position.total_cmp(&b.position));
            PlanEra {
                from: *from,
                to: bounds.get(index + 1).copied(),
                requirements,
            }
        })
        .collect();
    eras.reverse();
    eras
}

/// One past period reduced to how many of each plan's quotas were reached.
#[derive(Debug, Clone)]
pub struct PeriodOutcome {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub plans: Vec<PlanOutcome>,
}

/// `reached` of `total` counts the requirements of the lowest asking tier - the
/// ones the period passes or fails on.
#[derive(Debug, Clone)]
pub struct PlanOutcome {
    pub plan: Plan,
    pub reached: usize,
    pub total: usize,
    pub met: bool,
    pub medals: Vec<Medal>,
}

#[derive(Debug, Clone)]
pub struct PeriodProgress {
    pub period: Period,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub plans: Vec<PlanProgress>,
}

#[derive(Debug, Clone)]
pub struct PlanProgress {
    pub plan: Plan,
    pub items: Vec<RequirementProgress>,
    pub standing: Vec<TierStanding>,
    /// Consecutive periods completed, counting back from the current one. An
    /// unfinished current period does not break it.
    pub streak: u32,
}

#[derive(Debug, Clone)]
pub struct RequirementProgress {
    pub id: Uuid,
    pub name: Option<String>,
    pub habits: Vec<Habit>,
    pub quotas: Vec<TierQuota>,
    pub measure: Measure,
    pub done: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn version(from: &str, to: Option<&str>) -> Requirement {
        Requirement {
            id: Uuid::nil(),
            plan_id: Uuid::nil(),
            name: None,
            quotas: vec![TierQuota {
                tier_id: Uuid::nil(),
                quota: 1.0,
            }],
            measure: Measure::Occurrences,
            position: 0.0,
            habit_ids: Vec::new(),
            valid_from: d(from),
            valid_to: to.map(d),
        }
    }

    #[test]
    fn a_version_covers_its_periods_and_nothing_else() {
        let old = version("2026-09-07", Some("2026-09-21"));
        let new = version("2026-09-21", None);
        let weeks = [
            "2026-08-31",
            "2026-09-07",
            "2026-09-14",
            "2026-09-21",
            "2026-09-28",
        ];
        let covered: Vec<(bool, bool)> = weeks
            .iter()
            .map(|week| (old.in_force_at(d(week)), new.in_force_at(d(week))))
            .collect();
        assert_eq!(
            covered,
            [
                (false, false),
                (true, false),
                (true, false),
                (false, true),
                (false, true)
            ]
        );
    }

    #[test]
    fn eras_split_at_every_change_newest_first() {
        let mut quota_five = version("2026-09-07", Some("2026-09-21"));
        quota_five.quotas[0].quota = 5.0;
        let mut quota_six = version("2026-09-21", None);
        quota_six.quotas[0].quota = 6.0;
        let added_later = version("2026-09-28", None);
        let removed = version("2026-09-07", Some("2026-09-28"));

        let eras = eras(&[quota_five, quota_six, added_later, removed]);
        let shape: Vec<(String, Option<String>, Vec<f64>)> = eras
            .iter()
            .map(|era| {
                (
                    era.from.to_string(),
                    era.to.map(|to| to.to_string()),
                    era.requirements.iter().map(|r| r.quotas[0].quota).collect(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            [
                ("2026-09-28".into(), None, vec![6.0, 1.0]),
                (
                    "2026-09-21".into(),
                    Some("2026-09-28".into()),
                    vec![6.0, 1.0]
                ),
                (
                    "2026-09-07".into(),
                    Some("2026-09-21".into()),
                    vec![5.0, 1.0]
                ),
            ]
        );
    }

    #[test]
    fn removing_everything_leaves_an_empty_stretch() {
        let eras = eras(&[version("2026-09-07", Some("2026-09-14"))]);
        assert_eq!(eras.len(), 2);
        assert!(eras[0].requirements.is_empty());
        assert_eq!(eras[0].from, d("2026-09-14"));
    }

    fn tier(n: u128, medal: Medal) -> Tier {
        Tier {
            id: Uuid::from_u128(n),
            name: None,
            medal: Some(medal),
            position: n as f64,
            retired_on: None,
        }
    }

    fn goal(id: u128, quotas: &[(u128, f64)]) -> Requirement {
        Requirement {
            id: Uuid::from_u128(id),
            quotas: quotas
                .iter()
                .map(|(tier, quota)| TierQuota {
                    tier_id: Uuid::from_u128(*tier),
                    quota: *quota,
                })
                .collect(),
            ..version("2026-09-07", None)
        }
    }

    fn reached(standing: &[TierStanding]) -> Vec<bool> {
        standing.iter().map(|tier| tier.reached).collect()
    }

    /// The case settled in conversation: training skips the top tier, words skip the
    /// middle one. Clearing the top without the middle earns the bottom alone.
    #[test]
    fn tiers_are_climbed_in_order() {
        let tiers = [
            tier(1, Medal::Bronze),
            tier(2, Medal::Silver),
            tier(3, Medal::Gold),
        ];
        let training = goal(10, &[(1, 2.0), (2, 4.0)]);
        let words = goal(20, &[(1, 20.0), (3, 100.0)]);
        let done = |r: &Requirement| if r.id == training.id { 3.0 } else { 100.0 };

        let standing = standing(&tiers, &[training.clone(), words.clone()], done);
        assert_eq!(reached(&standing), [true, false, false]);
        assert!(met(&standing));
        assert_eq!(medals(&standing), [Medal::Bronze]);
    }

    #[test]
    fn every_tier_reached_earns_every_medal() {
        let tiers = [
            tier(1, Medal::Bronze),
            tier(2, Medal::Gold),
            tier(3, Medal::Gold),
        ];
        let training = goal(10, &[(1, 2.0), (2, 4.0), (3, 5.0)]);
        let standing = standing(&tiers, &[training], |_| 5.0);
        assert_eq!(medals(&standing), [Medal::Bronze, Medal::Gold, Medal::Gold]);
    }

    /// A tier no goal asks at is not a hurdle; it simply is not there that period.
    #[test]
    fn a_tier_nobody_asks_at_is_skipped() {
        let tiers = [
            tier(1, Medal::Bronze),
            tier(2, Medal::Silver),
            tier(3, Medal::Gold),
        ];
        let training = goal(10, &[(1, 2.0), (3, 5.0)]);
        let standing = standing(&tiers, &[training], |_| 5.0);
        assert_eq!(standing.len(), 2);
        assert_eq!(medals(&standing), [Medal::Bronze, Medal::Gold]);
    }

    /// A goal that only starts at a higher tier does not decide whether the period
    /// passes: the lowest asking tier does.
    #[test]
    fn a_goal_above_the_base_does_not_block_the_streak() {
        let tiers = [tier(1, Medal::Bronze), tier(2, Medal::Silver)];
        let training = goal(10, &[(1, 2.0)]);
        let reading = goal(20, &[(2, 3.0)]);
        let done = |r: &Requirement| if r.id == training.id { 2.0 } else { 0.0 };
        let standing = standing(&tiers, &[training.clone(), reading.clone()], done);
        assert!(met(&standing));
        assert_eq!(reached(&standing), [true, false]);
    }

    #[test]
    fn nothing_asked_is_not_met() {
        assert!(!met(&standing(&[tier(1, Medal::Bronze)], &[], |_| 0.0)));
    }

    #[test]
    fn an_archived_plan_keeps_the_periods_before_it() {
        let plan = Plan {
            id: Uuid::nil(),
            name: "minimum".into(),
            period: Period::Week,
            position: 0.0,
            archived_on: Some(d("2026-09-23")),
            tiers: Vec::new(),
        };
        assert!(plan.counts_before(d("2026-09-21")));
        assert!(!plan.counts_before(d("2026-09-28")));
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

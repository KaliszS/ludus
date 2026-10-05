use std::collections::HashMap;

use chrono::{Days, NaiveDate, Utc};
use uuid::Uuid;

use super::{UserService, parse_period, require_name};
use crate::domain::{
    Habit, LevelEra, LevelOutcome, LevelProgress, Measure, Period, PeriodOutcome, PlanLevel,
    PlanProgress, Requirement, RequirementProgress, eras,
};
use crate::error::{AppError, AppResult};
use crate::repo::plan::{LevelChanges, NewLevel, NewRequirement, RequirementChanges};
use crate::repo::{checkin, habit, plan};

/// How far back a level streak is counted. Anything longer reports as this.
const STREAK_LOOKBACK: u32 = 52;

/// Per habit: the summed amount and the number of days with any check-in.
type Bucket = HashMap<Uuid, (f64, u32)>;

/// Buckets every row into its window in one pass. Scanning the rows once per
/// window instead makes this O(rows x windows), which is what it used to be.
fn totals_per_window(
    rows: &[(Uuid, NaiveDate, i32)],
    windows: &[(NaiveDate, NaiveDate)],
) -> Vec<Bucket> {
    let mut buckets = vec![Bucket::new(); windows.len()];
    for (habit_id, day, times) in rows {
        let found = windows.binary_search_by(|(start, end)| {
            if day < start {
                std::cmp::Ordering::Greater
            } else if day >= end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        if let Ok(index) = found {
            let entry = buckets[index].entry(*habit_id).or_insert((0.0, 0));
            entry.0 += f64::from(*times);
            entry.1 += 1;
        }
    }
    buckets
}

/// Members are summed, so one member is a plain quota and several accept any mix.
fn done_for(requirement: &Requirement, bucket: &Bucket) -> f64 {
    requirement
        .habit_ids
        .iter()
        .filter_map(|habit_id| bucket.get(habit_id))
        .map(|(amount, days)| match requirement.measure {
            Measure::Amount => *amount,
            Measure::Occurrences => f64::from(*days),
        })
        .sum()
}

type Window = (NaiveDate, NaiveDate);

/// Habit id -> the day it was archived on. Absent means still active.
type Archived = HashMap<Uuid, NaiveDate>;

/// What a level asked of one period: the requirement versions in force when it
/// opened, members narrowed to habits not yet archived when it closed. Empty when
/// the level did not exist yet, or had been archived by then.
///
/// Archiving takes effect for the period it happens in, the same as an edit does,
/// so every period before it keeps the verdict it had.
fn asked_of(
    level: &PlanLevel,
    requirements: &[Requirement],
    (start, end): Window,
    archived: &Archived,
) -> Vec<Requirement> {
    if !level.counts_before(end) {
        return Vec::new();
    }
    requirements
        .iter()
        .filter(|requirement| requirement.level_id == level.id && requirement.in_force_at(start))
        .filter_map(|requirement| {
            let mut requirement = requirement.clone();
            requirement
                .habit_ids
                .retain(|habit_id| archived.get(habit_id).is_none_or(|day| *day >= end));
            (!requirement.habit_ids.is_empty()).then_some(requirement)
        })
        .collect()
}

fn level_met(asked: &[Requirement], bucket: &Bucket) -> bool {
    !asked.is_empty()
        && asked
            .iter()
            .all(|requirement| done_for(requirement, bucket) >= requirement.quota)
}

/// Consecutive periods met, newest first. Each period is judged by its own
/// versions, so a run carries straight across an edit. The current period adds to
/// it when met but never breaks it; a period that asked nothing ends it.
fn streak(
    level: &PlanLevel,
    requirements: &[Requirement],
    windows: &[Window],
    buckets: &[Bucket],
    archived: &Archived,
) -> u32 {
    let mut streak = 0;
    for (offset, (window, bucket)) in windows.iter().zip(buckets).rev().enumerate() {
        let asked = asked_of(level, requirements, *window, archived);
        if asked.is_empty() {
            break;
        }
        if level_met(&asked, bucket) {
            streak += 1;
        } else if offset > 0 {
            break;
        }
    }
    streak
}

/// The client's today decides which period an edit lands in, since that is the
/// period its screen shows. Clamped to a day either side of UTC: enough for any
/// time zone, too little to quietly rewrite last month.
fn effective_day(on: Option<NaiveDate>) -> NaiveDate {
    let today = Utc::now().date_naive();
    on.unwrap_or(today)
        .clamp(today - Days::new(1), today + Days::new(1))
}

fn same_members(a: &[Uuid], b: &[Uuid]) -> bool {
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

#[derive(Default)]
pub struct CreateLevel {
    pub name: String,
    pub period: Option<String>,
}

#[derive(Default)]
pub struct UpdateLevel {
    pub name: Option<String>,
    pub period: Option<String>,
    pub position: Option<f64>,
    pub archived: Option<bool>,
}

pub struct CreateRequirement {
    pub name: Option<String>,
    pub habit_ids: Vec<Uuid>,
    pub quota: f64,
    pub measure: Option<String>,
}

#[derive(Default)]
pub struct UpdateRequirement {
    pub name: Option<Option<String>>,
    pub habit_ids: Option<Vec<Uuid>>,
    pub quota: Option<f64>,
    pub measure: Option<String>,
    pub position: Option<f64>,
}

fn parse_measure(value: &str) -> AppResult<Measure> {
    Measure::parse(value)
        .ok_or_else(|| AppError::invalid("measure", r#"must be "amount" or "occurrences""#))
}

impl UserService {
    pub async fn plan_levels(
        &self,
        period: Option<Period>,
        include_archived: bool,
    ) -> AppResult<Vec<PlanLevel>> {
        let mut conn = self.conn().await?;
        plan::list_levels(&mut conn, self.user_id, period, include_archived).await
    }

    pub async fn plan_level(&self, id: Uuid) -> AppResult<PlanLevel> {
        let mut conn = self.conn().await?;
        plan::get_level(&mut conn, self.user_id, id).await
    }

    pub async fn create_plan_level(&self, input: CreateLevel) -> AppResult<PlanLevel> {
        let name = require_name("name", &input.name)?;
        let period = match input.period.as_deref() {
            Some(value) => parse_period(value)?,
            None => Period::Week,
        };

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let position = plan::next_level_position(&mut conn, user_id, period).await?;

        plan::insert_level(
            &mut conn,
            NewLevel {
                id: Uuid::new_v4(),
                user_id,
                name,
                period: period.as_str().to_owned(),
                position,
            },
        )
        .await
    }

    /// Archiving stops the level counting from the current period on and keeps every
    /// earlier one. Switching period is only allowed before there is any history,
    /// since afterwards it would re-cut every past period into a different shape.
    pub async fn update_plan_level(
        &self,
        id: Uuid,
        input: UpdateLevel,
        on: Option<NaiveDate>,
    ) -> AppResult<PlanLevel> {
        let name = input
            .name
            .as_deref()
            .map(|value| require_name("name", value))
            .transpose()?;
        let period = input.period.as_deref().map(parse_period).transpose()?;
        let day = effective_day(on);

        let mut conn = self.conn().await?;
        let level = plan::get_level(&mut conn, self.user_id, id).await?;
        let switch = period.filter(|period| *period != level.period);
        if let Some(to) = switch
            && plan::level_has_history(&mut conn, id, level.period.window(day).0).await?
        {
            let (from, to) = (level.period.as_str(), to.as_str());
            return Err(AppError::invalid(
                "period",
                format!(
                    "Its past {from}s were already judged as {from}s and cannot be re-cut into \
                     {to}s. To switch, archive this level - its {from}s stay in your history - \
                     and add a new {to} level."
                ),
            ));
        }

        let updated = plan::update_level(
            &mut conn,
            self.user_id,
            id,
            LevelChanges {
                name,
                period: switch.map(|period| period.as_str().to_owned()),
                position: input.position,
                archived_on: input.archived.map(|archived| archived.then_some(day)),
            },
        )
        .await?;
        if let Some(period) = switch {
            plan::realign_requirements(&mut conn, id, period.window(day).0).await?;
        }
        Ok(updated)
    }

    /// Erases the level and its whole history. Archiving is the way to keep it.
    pub async fn delete_plan_level(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        plan::delete_level(&mut conn, self.user_id, id).await
    }

    /// Every shape the level has had, newest first, with the habits it named - archived
    /// ones included, since an old version may well point at them.
    pub async fn plan_versions(
        &self,
        level_id: Uuid,
    ) -> AppResult<(PlanLevel, Vec<LevelEra>, Vec<Habit>)> {
        let mut conn = self.conn().await?;
        let level = plan::get_level(&mut conn, self.user_id, level_id).await?;
        let versions = plan::requirements_for(&mut conn, &[level_id]).await?;
        let mut habit_ids: Vec<Uuid> = versions
            .iter()
            .flat_map(|version| version.habit_ids.iter().copied())
            .collect();
        habit_ids.sort_unstable();
        habit_ids.dedup();
        let habits = habit::by_ids(&mut conn, self.user_id, &habit_ids).await?;
        Ok((level, eras(&versions), habits))
    }

    /// The versions in force now; earlier ones only matter to history.
    pub async fn requirements(&self, level_id: Uuid) -> AppResult<Vec<Requirement>> {
        let mut conn = self.conn().await?;
        plan::get_level(&mut conn, self.user_id, level_id).await?;
        Ok(plan::requirements_for(&mut conn, &[level_id])
            .await?
            .into_iter()
            .filter(|requirement| requirement.valid_to.is_none())
            .collect())
    }

    pub async fn create_requirement(
        &self,
        level_id: Uuid,
        input: CreateRequirement,
        on: Option<NaiveDate>,
    ) -> AppResult<Requirement> {
        let measure = match input.measure.as_deref() {
            Some(value) => parse_measure(value)?,
            None => Measure::Amount,
        };
        if input.quota <= 0.0 {
            return Err(AppError::invalid("quota", "must be positive"));
        }

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let level = plan::get_level(&mut conn, user_id, level_id).await?;
        let habit_ids = self
            .checked_habits(&mut conn, user_id, &input.habit_ids)
            .await?;

        let name = input
            .name
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let position = plan::next_requirement_position(&mut conn, level_id).await?;
        let valid_from = level.period.window(effective_day(on)).0;
        let id = plan::insert_requirement(
            &mut conn,
            NewRequirement {
                id: Uuid::new_v4(),
                level_id,
                name: name.clone(),
                quota: input.quota,
                measure: measure.as_str().to_owned(),
                position,
                valid_from,
            },
            &habit_ids,
        )
        .await?;

        Ok(Requirement {
            id,
            level_id,
            name,
            quota: input.quota,
            measure,
            position,
            habit_ids,
            valid_from,
            valid_to: None,
        })
    }

    /// A different quota, measure or set of habits changes what a period asks for.
    /// When the version in force already covered earlier periods, it is closed and a
    /// new one opens with the current period, so the earlier periods keep their
    /// verdict. Anything else - a label, the order, or a version that only covers
    /// this period - is edited where it stands.
    pub async fn update_requirement(
        &self,
        id: Uuid,
        input: UpdateRequirement,
        on: Option<NaiveDate>,
    ) -> AppResult<()> {
        let measure = input.measure.as_deref().map(parse_measure).transpose()?;
        if let Some(quota) = input.quota
            && quota <= 0.0
        {
            return Err(AppError::invalid("quota", "must be positive"));
        }

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let level_id = plan::requirement_level(&mut conn, user_id, id).await?;
        let level = plan::get_level(&mut conn, user_id, level_id).await?;
        let current = plan::requirements_for(&mut conn, &[level_id])
            .await?
            .into_iter()
            .find(|requirement| requirement.id == id)
            .ok_or(AppError::NotFound)?;
        let habit_ids = match input.habit_ids {
            Some(ids) => Some(self.checked_habits(&mut conn, user_id, &ids).await?),
            None => None,
        };
        let name = input
            .name
            .map(|value| value.and_then(|v| Some(v.trim().to_owned()).filter(|v| !v.is_empty())));

        let reshapes = input.quota.is_some_and(|quota| quota != current.quota)
            || measure.is_some_and(|measure| measure != current.measure)
            || habit_ids
                .as_deref()
                .is_some_and(|ids| !same_members(ids, &current.habit_ids));
        let start = level.period.window(effective_day(on)).0;

        if reshapes && current.valid_from < start {
            plan::replace_requirement(
                &mut conn,
                id,
                NewRequirement {
                    id: Uuid::new_v4(),
                    level_id,
                    name: name.unwrap_or(current.name),
                    quota: input.quota.unwrap_or(current.quota),
                    measure: measure.unwrap_or(current.measure).as_str().to_owned(),
                    position: input.position.unwrap_or(current.position),
                    valid_from: start,
                },
                habit_ids.as_deref().unwrap_or(&current.habit_ids),
            )
            .await?;
            return Ok(());
        }

        plan::update_requirement(
            &mut conn,
            id,
            RequirementChanges {
                name,
                quota: input.quota,
                measure: measure.map(|measure| measure.as_str().to_owned()),
                position: input.position,
            },
            habit_ids.as_deref(),
        )
        .await
    }

    /// Removes a requirement from the current period on. One that already covered
    /// earlier periods is closed rather than deleted, so those periods stay as they were.
    pub async fn delete_requirement(&self, id: Uuid, on: Option<NaiveDate>) -> AppResult<()> {
        let mut conn = self.conn().await?;
        let level_id = plan::requirement_level(&mut conn, self.user_id, id).await?;
        let level = plan::get_level(&mut conn, self.user_id, level_id).await?;
        let current = plan::requirements_for(&mut conn, &[level_id])
            .await?
            .into_iter()
            .find(|requirement| requirement.id == id)
            .ok_or(AppError::NotFound)?;

        let start = level.period.window(effective_day(on)).0;
        if current.valid_from < start {
            plan::close_requirement(&mut conn, id, start).await
        } else {
            plan::delete_requirement(&mut conn, id).await
        }
    }

    /// Rejects a requirement that names a habit the caller does not own.
    async fn checked_habits(
        &self,
        conn: &mut crate::repo::pool::DbConn,
        user_id: Uuid,
        habit_ids: &[Uuid],
    ) -> AppResult<Vec<Uuid>> {
        let mut ids = habit_ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return Err(AppError::invalid(
                "habit_ids",
                "must name at least one habit",
            ));
        }

        let found = habit::by_ids(conn, user_id, &ids).await?;
        if found.len() != ids.len() {
            return Err(AppError::invalid(
                "habit_ids",
                "references a habit that does not exist",
            ));
        }
        Ok(ids)
    }

    /// How each level fared over the last `count` periods, archived levels included
    /// for the periods before they were archived. Check-ins for the whole span are
    /// fetched once and bucketed here, so the cost does not grow with `count`.
    pub async fn plan_history(
        &self,
        period: Period,
        count: u32,
        on: NaiveDate,
    ) -> AppResult<Vec<PeriodOutcome>> {
        let count = count.clamp(1, 53);
        let user_id = self.user_id;
        let mut conn = self.conn().await?;

        let levels = plan::list_levels(&mut conn, user_id, Some(period), true).await?;
        if levels.is_empty() {
            return Ok(Vec::new());
        }

        let windows: Vec<Window> = (0..count)
            .rev()
            .map(|back| period.window(period.step_back(on, back)))
            .collect();

        let level_ids: Vec<Uuid> = levels.iter().map(|level| level.id).collect();
        let requirements = plan::requirements_for(&mut conn, &level_ids).await?;
        let archived = habit::archived_days(&mut conn, user_id).await?;
        let rows = checkin::range_for_user(
            &mut conn,
            user_id,
            windows[0].0,
            windows[count as usize - 1].1,
        )
        .await?;
        let buckets = totals_per_window(&rows, &windows);

        Ok(windows
            .into_iter()
            .zip(buckets)
            .map(|(window, bucket)| PeriodOutcome {
                period_start: window.0,
                period_end: window.1,
                levels: levels
                    .iter()
                    .filter_map(|level| {
                        let asked = asked_of(level, &requirements, window, &archived);
                        (!asked.is_empty()).then(|| LevelOutcome {
                            level: level.clone(),
                            reached: asked
                                .iter()
                                .filter(|requirement| {
                                    done_for(requirement, &bucket) >= requirement.quota
                                })
                                .count(),
                            total: asked.len(),
                        })
                    })
                    .collect(),
            })
            .collect())
    }

    /// The plan screen: every level of one period with what it asks now, how far
    /// along it is, and its streak.
    pub async fn plan_progress(&self, period: Period, on: NaiveDate) -> AppResult<PlanProgress> {
        let (period_start, period_end) = period.window(on);
        let user_id = self.user_id;
        let mut conn = self.conn().await?;

        let levels = plan::list_levels(&mut conn, user_id, Some(period), false).await?;
        if levels.is_empty() {
            return Ok(PlanProgress {
                period,
                period_start,
                period_end,
                levels: Vec::new(),
            });
        }

        let level_ids: Vec<Uuid> = levels.iter().map(|level| level.id).collect();
        let requirements = plan::requirements_for(&mut conn, &level_ids).await?;
        let archived = habit::archived_days(&mut conn, user_id).await?;

        // One fetch covers both the current window and the streak lookback.
        let windows: Vec<Window> = (0..STREAK_LOOKBACK)
            .rev()
            .map(|back| period.window(period.step_back(on, back)))
            .collect();
        let rows = checkin::range_for_user(&mut conn, user_id, windows[0].0, period_end).await?;
        let buckets = totals_per_window(&rows, &windows);
        let now = (period_start, period_end);
        let current = buckets.last().cloned().unwrap_or_default();

        let asked: Vec<(PlanLevel, Vec<Requirement>)> = levels
            .into_iter()
            .map(|level| {
                let asked = asked_of(&level, &requirements, now, &archived);
                (level, asked)
            })
            .filter(|(_, asked)| !asked.is_empty())
            .collect();

        let mut habit_ids: Vec<Uuid> = asked
            .iter()
            .flat_map(|(_, asked)| {
                asked
                    .iter()
                    .flat_map(|requirement| requirement.habit_ids.iter().copied())
            })
            .collect();
        habit_ids.sort_unstable();
        habit_ids.dedup();
        let habits: HashMap<Uuid, _> = habit::by_ids(&mut conn, user_id, &habit_ids)
            .await?
            .into_iter()
            .map(|habit| (habit.id, habit))
            .collect();

        Ok(PlanProgress {
            period,
            period_start,
            period_end,
            levels: asked
                .into_iter()
                .map(|(level, asked)| LevelProgress {
                    streak: streak(&level, &requirements, &windows, &buckets, &archived),
                    items: asked
                        .iter()
                        .map(|requirement| RequirementProgress {
                            id: requirement.id,
                            name: requirement.name.clone(),
                            habits: requirement
                                .habit_ids
                                .iter()
                                .filter_map(|habit_id| habits.get(habit_id).cloned())
                                .collect(),
                            quota: requirement.quota,
                            measure: requirement.measure,
                            done: done_for(requirement, &current),
                        })
                        .collect(),
                    level,
                })
                .collect(),
        })
    }
}

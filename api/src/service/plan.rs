use std::collections::HashMap;

use chrono::NaiveDate;
use uuid::Uuid;

use super::{Service, parse_period, require_name};
use crate::domain::{
    LevelOutcome, LevelProgress, Measure, Period, PeriodOutcome, PlanLevel, PlanProgress,
    Requirement, RequirementProgress,
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

fn level_met(requirements: &[&Requirement], bucket: &Bucket) -> bool {
    !requirements.is_empty()
        && requirements
            .iter()
            .all(|requirement| done_for(requirement, bucket) >= requirement.quota)
}

/// An archived habit is hidden everywhere, so it must not keep a plan alive either.
/// Members go first, then requirements left empty, then levels left with nothing.
fn restrict_to_active(requirements: Vec<Requirement>, active: &[Uuid]) -> Vec<Requirement> {
    requirements
        .into_iter()
        .filter_map(|mut requirement| {
            requirement
                .habit_ids
                .retain(|habit_id| active.contains(habit_id));
            (!requirement.habit_ids.is_empty()).then_some(requirement)
        })
        .collect()
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

impl Service {
    pub async fn plan_levels(&self, period: Option<Period>) -> AppResult<Vec<PlanLevel>> {
        let mut conn = self.conn().await?;
        plan::list_levels(&mut conn, self.current_user(), period).await
    }

    pub async fn plan_level(&self, id: Uuid) -> AppResult<PlanLevel> {
        let mut conn = self.conn().await?;
        plan::get_level(&mut conn, self.current_user(), id).await
    }

    pub async fn create_plan_level(&self, input: CreateLevel) -> AppResult<PlanLevel> {
        let name = require_name("name", &input.name)?;
        let period = match input.period.as_deref() {
            Some(value) => parse_period(value)?,
            None => Period::Week,
        };

        let user_id = self.current_user();
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

    pub async fn update_plan_level(&self, id: Uuid, input: UpdateLevel) -> AppResult<PlanLevel> {
        let name = input
            .name
            .as_deref()
            .map(|value| require_name("name", value))
            .transpose()?;
        let period = input
            .period
            .as_deref()
            .map(parse_period)
            .transpose()?
            .map(|period| period.as_str().to_owned());

        let mut conn = self.conn().await?;
        plan::update_level(
            &mut conn,
            self.current_user(),
            id,
            LevelChanges {
                name,
                period,
                position: input.position,
            },
        )
        .await
    }

    pub async fn delete_plan_level(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        plan::delete_level(&mut conn, self.current_user(), id).await
    }

    pub async fn requirements(&self, level_id: Uuid) -> AppResult<Vec<Requirement>> {
        let mut conn = self.conn().await?;
        plan::get_level(&mut conn, self.current_user(), level_id).await?;
        plan::requirements_for(&mut conn, &[level_id]).await
    }

    pub async fn create_requirement(
        &self,
        level_id: Uuid,
        input: CreateRequirement,
    ) -> AppResult<Requirement> {
        let measure = match input.measure.as_deref() {
            Some(value) => parse_measure(value)?,
            None => Measure::Amount,
        };
        if input.quota <= 0.0 {
            return Err(AppError::invalid("quota", "must be positive"));
        }

        let user_id = self.current_user();
        let mut conn = self.conn().await?;
        plan::get_level(&mut conn, user_id, level_id).await?;
        let habit_ids = self
            .checked_habits(&mut conn, user_id, &input.habit_ids)
            .await?;

        let name = input
            .name
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let position = plan::next_requirement_position(&mut conn, level_id).await?;
        let id = plan::insert_requirement(
            &mut conn,
            NewRequirement {
                id: Uuid::new_v4(),
                level_id,
                name: name.clone(),
                quota: input.quota,
                measure: measure.as_str().to_owned(),
                position,
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
        })
    }

    pub async fn update_requirement(&self, id: Uuid, input: UpdateRequirement) -> AppResult<()> {
        let measure = input
            .measure
            .as_deref()
            .map(parse_measure)
            .transpose()?
            .map(|measure| measure.as_str().to_owned());
        if let Some(quota) = input.quota
            && quota <= 0.0
        {
            return Err(AppError::invalid("quota", "must be positive"));
        }

        let user_id = self.current_user();
        let mut conn = self.conn().await?;
        plan::requirement_level(&mut conn, user_id, id).await?;

        let habit_ids = match input.habit_ids {
            Some(ids) => Some(self.checked_habits(&mut conn, user_id, &ids).await?),
            None => None,
        };

        plan::update_requirement(
            &mut conn,
            id,
            RequirementChanges {
                name: input.name.map(|value| {
                    value.and_then(|v| Some(v.trim().to_owned()).filter(|v| !v.is_empty()))
                }),
                quota: input.quota,
                measure,
                position: input.position,
            },
            habit_ids.as_deref(),
        )
        .await
    }

    pub async fn delete_requirement(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        plan::requirement_level(&mut conn, self.current_user(), id).await?;
        plan::delete_requirement(&mut conn, id).await
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

    /// How each level fared over the last `count` periods. Check-ins for the whole span
    /// are fetched once and bucketed here, so the cost does not grow with `count`.
    pub async fn plan_history(
        &self,
        period: Period,
        count: u32,
        on: NaiveDate,
    ) -> AppResult<Vec<PeriodOutcome>> {
        let count = count.clamp(1, 53);
        let user_id = self.current_user();
        let mut conn = self.conn().await?;

        let levels = plan::list_levels(&mut conn, user_id, Some(period)).await?;
        if levels.is_empty() {
            return Ok(Vec::new());
        }

        let windows: Vec<(NaiveDate, NaiveDate)> = (0..count)
            .rev()
            .map(|back| period.window(period.step_back(on, back)))
            .collect();

        let level_ids: Vec<Uuid> = levels.iter().map(|level| level.id).collect();
        let active = habit::active_ids(&mut conn, user_id).await?;
        let requirements = restrict_to_active(
            plan::requirements_for(&mut conn, &level_ids).await?,
            &active,
        );
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
            .map(|((start, end), bucket)| PeriodOutcome {
                period_start: start,
                period_end: end,
                levels: levels
                    .iter()
                    .filter(|level| {
                        requirements
                            .iter()
                            .any(|requirement| requirement.level_id == level.id)
                    })
                    .map(|level| {
                        let items: Vec<&Requirement> = requirements
                            .iter()
                            .filter(|requirement| requirement.level_id == level.id)
                            .collect();
                        LevelOutcome {
                            level: level.clone(),
                            reached: items
                                .iter()
                                .filter(|requirement| {
                                    done_for(requirement, &bucket) >= requirement.quota
                                })
                                .count(),
                            total: items.len(),
                        }
                    })
                    .collect(),
            })
            .collect())
    }

    /// The plan screen: every level of one period with its requirements and how far along they are.
    pub async fn plan_progress(&self, period: Period, on: NaiveDate) -> AppResult<PlanProgress> {
        let (period_start, period_end) = period.window(on);
        let user_id = self.current_user();
        let mut conn = self.conn().await?;

        let levels = plan::list_levels(&mut conn, user_id, Some(period)).await?;
        if levels.is_empty() {
            return Ok(PlanProgress {
                period,
                period_start,
                period_end,
                levels: Vec::new(),
            });
        }

        let level_ids: Vec<Uuid> = levels.iter().map(|level| level.id).collect();
        let active = habit::active_ids(&mut conn, user_id).await?;
        let requirements = restrict_to_active(
            plan::requirements_for(&mut conn, &level_ids).await?,
            &active,
        );

        let mut habit_ids: Vec<Uuid> = requirements
            .iter()
            .flat_map(|requirement| requirement.habit_ids.iter().copied())
            .collect();
        habit_ids.sort_unstable();
        habit_ids.dedup();

        let habits: HashMap<Uuid, _> = habit::by_ids(&mut conn, user_id, &habit_ids)
            .await?
            .into_iter()
            .map(|habit| (habit.id, habit))
            .collect();

        // One fetch covers both the current window and the streak lookback.
        let windows: Vec<(NaiveDate, NaiveDate)> = (0..STREAK_LOOKBACK)
            .rev()
            .map(|back| period.window(period.step_back(on, back)))
            .collect();
        let rows = checkin::range_for_user(&mut conn, user_id, windows[0].0, period_end).await?;
        let buckets = totals_per_window(&rows, &windows);
        let current = buckets.last().cloned().unwrap_or_default();

        Ok(PlanProgress {
            period,
            period_start,
            period_end,
            levels: levels
                .into_iter()
                .filter(|level| {
                    requirements
                        .iter()
                        .any(|requirement| requirement.level_id == level.id)
                })
                .map(|level| {
                    let items: Vec<&Requirement> = requirements
                        .iter()
                        .filter(|requirement| requirement.level_id == level.id)
                        .collect();

                    let mut streak = 0;
                    for (offset, bucket) in buckets.iter().rev().enumerate() {
                        if level_met(&items, bucket) {
                            streak += 1;
                        } else if offset > 0 {
                            break;
                        }
                    }

                    LevelProgress {
                        items: items
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
                        streak,
                    }
                })
                .collect(),
        })
    }
}

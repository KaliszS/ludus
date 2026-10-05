use std::collections::HashMap;

use chrono::{Days, NaiveDate, Utc};
use uuid::Uuid;

use super::{UserService, parse_period, require_name};
use crate::domain::plan::{medals, met, standing};
use crate::domain::{
    Habit, Measure, Medal, Period, PeriodOutcome, PeriodProgress, Plan, PlanEra, PlanOutcome,
    PlanProgress, Requirement, RequirementProgress, Tier, TierQuota, TierStanding, eras,
};
use crate::error::{AppError, AppResult};
use crate::repo::plan::{
    NewPlan, NewRequirement, NewTier, PlanChanges, RequirementChanges, TierChanges,
};
use crate::repo::pool::DbConn;
use crate::repo::{checkin, habit, plan};

/// How far back a plan streak is counted. Anything longer reports as this.
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

/// What a plan asked of one period: the requirement versions in force when it
/// opened, members narrowed to habits not yet archived when it closed. Empty when
/// the plan did not exist yet, or had been archived by then.
///
/// Archiving takes effect for the period it happens in, the same as an edit does,
/// so every period before it keeps the verdict it had.
fn asked_of(
    plan: &Plan,
    requirements: &[Requirement],
    (start, end): Window,
    archived: &Archived,
) -> Vec<Requirement> {
    if !plan.counts_before(end) {
        return Vec::new();
    }
    requirements
        .iter()
        .filter(|requirement| requirement.plan_id == plan.id && requirement.in_force_at(start))
        .filter_map(|requirement| {
            let mut requirement = requirement.clone();
            requirement
                .habit_ids
                .retain(|habit_id| archived.get(habit_id).is_none_or(|day| *day >= end));
            (!requirement.habit_ids.is_empty()).then_some(requirement)
        })
        .collect()
}

/// How far up its tiers a plan got in one period.
fn climb(plan: &Plan, asked: &[Requirement], bucket: &Bucket) -> Vec<TierStanding> {
    standing(&plan.tiers, asked, |requirement| {
        done_for(requirement, bucket)
    })
}

/// Consecutive periods met, newest first. Each period is judged by its own
/// versions, so a run carries straight across an edit. The current period adds to
/// it when met but never breaks it; a period that asked nothing ends it.
fn streak(
    plan: &Plan,
    requirements: &[Requirement],
    windows: &[Window],
    buckets: &[Bucket],
    archived: &Archived,
) -> u32 {
    let mut streak = 0;
    for (offset, (window, bucket)) in windows.iter().zip(buckets).rev().enumerate() {
        let asked = asked_of(plan, requirements, *window, archived);
        if asked.is_empty() {
            break;
        }
        if met(&climb(plan, &asked, bucket)) {
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
pub struct CreatePlan {
    pub name: String,
    pub period: Option<String>,
}

#[derive(Default)]
pub struct UpdatePlan {
    pub name: Option<String>,
    pub period: Option<String>,
    pub position: Option<f64>,
    pub archived: Option<bool>,
}

pub struct CreateTier {
    pub name: Option<String>,
    pub medal: Option<String>,
    /// Slot it in right above this tier; without it, it goes on top.
    pub after: Option<Uuid>,
}

#[derive(Default)]
pub struct UpdateTier {
    pub name: Option<Option<String>>,
    pub medal: Option<Option<String>>,
}

pub struct QuotaInput {
    pub tier_id: Uuid,
    pub quota: f64,
}

pub struct CreateRequirement {
    pub name: Option<String>,
    pub habit_ids: Vec<Uuid>,
    pub quotas: Vec<QuotaInput>,
    pub measure: Option<String>,
}

#[derive(Default)]
pub struct UpdateRequirement {
    pub name: Option<Option<String>>,
    pub habit_ids: Option<Vec<Uuid>>,
    /// The full set: tiers left out are skipped from now on.
    pub quotas: Option<Vec<QuotaInput>>,
    pub measure: Option<String>,
    pub position: Option<f64>,
}

fn parse_measure(value: &str) -> AppResult<Measure> {
    Measure::parse(value)
        .ok_or_else(|| AppError::invalid("measure", r#"must be "amount" or "occurrences""#))
}

fn parse_medal(value: &str) -> AppResult<String> {
    Medal::parse(value)
        .map(|medal| medal.as_str().to_owned())
        .ok_or_else(|| AppError::invalid("medal", r#"must be "bronze", "silver" or "gold""#))
}

fn optional_name(value: String) -> Option<String> {
    Some(value.trim().to_owned()).filter(|value| !value.is_empty())
}

fn live_tiers(plan: &Plan) -> Vec<&Tier> {
    plan.tiers
        .iter()
        .filter(|tier| tier.retired_on.is_none())
        .collect()
}

/// Quotas name live tiers of this plan, each at most once, and rise with the tier:
/// a goal may skip tiers, but never ask less further up. Returned in tier order.
fn checked_quotas(plan: &Plan, input: &[QuotaInput]) -> AppResult<Vec<TierQuota>> {
    let invalid = |message: &str| Err(AppError::invalid("quotas", message));
    let live = live_tiers(plan);
    if input
        .iter()
        .any(|quota| !live.iter().any(|tier| tier.id == quota.tier_id))
    {
        return invalid("A quota names a tier this plan does not have.");
    }

    let mut quotas = Vec::new();
    for tier in live {
        let mut given = input.iter().filter(|quota| quota.tier_id == tier.id);
        if let Some(quota) = given.next() {
            if given.next().is_some() {
                return invalid("Give each tier at most one quota.");
            }
            if quota.quota <= 0.0 {
                return invalid("Quotas must be above zero.");
            }
            quotas.push(TierQuota {
                tier_id: tier.id,
                quota: quota.quota,
            });
        }
    }
    if quotas.is_empty() {
        return invalid("Set a quota for at least one tier.");
    }
    if quotas.windows(2).any(|pair| pair[1].quota <= pair[0].quota) {
        return invalid("Each tier has to ask for more than the tiers below it.");
    }
    Ok(quotas)
}

impl UserService {
    pub async fn plans(
        &self,
        period: Option<Period>,
        include_archived: bool,
    ) -> AppResult<Vec<Plan>> {
        let mut conn = self.conn().await?;
        plan::list(&mut conn, self.user_id, period, include_archived).await
    }

    pub async fn plan(&self, id: Uuid) -> AppResult<Plan> {
        let mut conn = self.conn().await?;
        plan::get(&mut conn, self.user_id, id).await
    }

    pub async fn create_plan(&self, input: CreatePlan) -> AppResult<Plan> {
        let name = require_name("name", &input.name)?;
        let period = match input.period.as_deref() {
            Some(value) => parse_period(value)?,
            None => Period::Week,
        };

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let position = plan::next_position(&mut conn, user_id, period).await?;
        let id = Uuid::new_v4();

        plan::insert(
            &mut conn,
            NewPlan {
                id,
                user_id,
                name,
                period: period.as_str().to_owned(),
                position,
            },
            NewTier {
                id: Uuid::new_v4(),
                plan_id: id,
                name: None,
                medal: None,
                position: 100.0,
            },
        )
        .await
    }

    /// Archiving stops the plan counting from the current period on and keeps every
    /// earlier one. Switching period is only allowed before there is any history,
    /// since afterwards it would re-cut every past period into a different shape.
    pub async fn update_plan(
        &self,
        id: Uuid,
        input: UpdatePlan,
        on: Option<NaiveDate>,
    ) -> AppResult<Plan> {
        let name = input
            .name
            .as_deref()
            .map(|value| require_name("name", value))
            .transpose()?;
        let period = input.period.as_deref().map(parse_period).transpose()?;
        let day = effective_day(on);

        let mut conn = self.conn().await?;
        let current = plan::get(&mut conn, self.user_id, id).await?;
        let switch = period.filter(|period| *period != current.period);
        if let Some(to) = switch
            && plan::has_history(&mut conn, id, current.period.window(day).0).await?
        {
            let (from, to) = (current.period.as_str(), to.as_str());
            return Err(AppError::invalid(
                "period",
                format!(
                    "Its past {from}s were already judged as {from}s and cannot be re-cut into \
                     {to}s. To switch, archive this plan - its {from}s stay in your history - \
                     and add a new {to} plan."
                ),
            ));
        }

        let updated = plan::update(
            &mut conn,
            self.user_id,
            id,
            PlanChanges {
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

    /// Erases the plan and its whole history. Archiving is the way to keep it.
    pub async fn delete_plan(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        plan::delete(&mut conn, self.user_id, id).await
    }

    /// A new tier asks nothing until quotas are set for it, so adding one never
    /// changes a past verdict.
    pub async fn create_tier(&self, plan_id: Uuid, input: CreateTier) -> AppResult<Plan> {
        let medal = input.medal.as_deref().map(parse_medal).transpose()?;
        let mut conn = self.conn().await?;
        let current = plan::get(&mut conn, self.user_id, plan_id).await?;
        let live = live_tiers(&current);
        let position = match input.after {
            None => live.last().map_or(100.0, |tier| tier.position + 100.0),
            Some(after) => {
                let index = live
                    .iter()
                    .position(|tier| tier.id == after)
                    .ok_or_else(|| AppError::invalid("after", "is not a tier of this plan"))?;
                let below = live[index].position;
                live.get(index + 1)
                    .map_or(below + 100.0, |above| (below + above.position) / 2.0)
            }
        };
        plan::insert_tier(
            &mut conn,
            NewTier {
                id: Uuid::new_v4(),
                plan_id,
                name: input.name.and_then(optional_name),
                medal,
                position,
            },
        )
        .await?;
        plan::get(&mut conn, self.user_id, plan_id).await
    }

    /// Name and medal are labels, not rules: changing them relabels the past too.
    pub async fn update_tier(&self, tier_id: Uuid, input: UpdateTier) -> AppResult<Plan> {
        let medal = match input.medal {
            Some(Some(value)) => Some(Some(parse_medal(&value)?)),
            Some(None) => Some(None),
            None => None,
        };
        let mut conn = self.conn().await?;
        let plan_id = plan::tier_plan(&mut conn, self.user_id, tier_id).await?;
        plan::update_tier(
            &mut conn,
            tier_id,
            TierChanges {
                name: input.name.map(|name| name.and_then(optional_name)),
                medal,
                retired_on: None,
            },
        )
        .await?;
        plan::get(&mut conn, self.user_id, plan_id).await
    }

    /// Retires a tier from the current period on. Requirements stop asking at it the
    /// same way any other edit works - a new version where the old one reaches back -
    /// and one that asked only at this tier ends. Past periods keep their quotas and
    /// medals, which is why the tier itself stays.
    pub async fn retire_tier(&self, tier_id: Uuid, on: Option<NaiveDate>) -> AppResult<Plan> {
        let mut conn = self.conn().await?;
        let plan_id = plan::tier_plan(&mut conn, self.user_id, tier_id).await?;
        let current = plan::get(&mut conn, self.user_id, plan_id).await?;
        if live_tiers(&current).len() <= 1 {
            return Err(AppError::invalid("tier", "A plan keeps at least one tier."));
        }

        let day = effective_day(on);
        let start = current.period.window(day).0;
        let asking: Vec<Requirement> = plan::requirements_for(&mut conn, &[plan_id])
            .await?
            .into_iter()
            .filter(|requirement| {
                requirement.valid_to.is_none() && requirement.quota_at(tier_id).is_some()
            })
            .collect();
        for requirement in asking {
            let rest: Vec<TierQuota> = requirement
                .quotas
                .iter()
                .copied()
                .filter(|quota| quota.tier_id != tier_id)
                .collect();
            if rest.is_empty() {
                end(&mut conn, &requirement, start).await?;
            } else if requirement.valid_from < start {
                plan::replace_requirement(
                    &mut conn,
                    requirement.id,
                    successor(&requirement, start),
                    &requirement.habit_ids,
                    &rest,
                )
                .await?;
            } else {
                plan::update_requirement(
                    &mut conn,
                    requirement.id,
                    RequirementChanges::default(),
                    None,
                    Some(&rest),
                )
                .await?;
            }
        }

        plan::update_tier(
            &mut conn,
            tier_id,
            TierChanges {
                retired_on: Some(Some(day)),
                ..TierChanges::default()
            },
        )
        .await?;
        plan::get(&mut conn, self.user_id, plan_id).await
    }

    /// Every shape the plan has had, newest first, with the habits it named - archived
    /// ones included, since an old version may well point at them.
    pub async fn plan_versions(
        &self,
        plan_id: Uuid,
    ) -> AppResult<(Plan, Vec<PlanEra>, Vec<Habit>)> {
        let mut conn = self.conn().await?;
        let current = plan::get(&mut conn, self.user_id, plan_id).await?;
        let versions = plan::requirements_for(&mut conn, &[plan_id]).await?;
        let mut habit_ids: Vec<Uuid> = versions
            .iter()
            .flat_map(|version| version.habit_ids.iter().copied())
            .collect();
        habit_ids.sort_unstable();
        habit_ids.dedup();
        let habits = habit::by_ids(&mut conn, self.user_id, &habit_ids).await?;
        Ok((current, eras(&versions), habits))
    }

    /// The versions in force now; earlier ones only matter to history.
    pub async fn requirements(&self, plan_id: Uuid) -> AppResult<Vec<Requirement>> {
        let mut conn = self.conn().await?;
        plan::get(&mut conn, self.user_id, plan_id).await?;
        Ok(plan::requirements_for(&mut conn, &[plan_id])
            .await?
            .into_iter()
            .filter(|requirement| requirement.valid_to.is_none())
            .collect())
    }

    pub async fn create_requirement(
        &self,
        plan_id: Uuid,
        input: CreateRequirement,
        on: Option<NaiveDate>,
    ) -> AppResult<Requirement> {
        let measure = match input.measure.as_deref() {
            Some(value) => parse_measure(value)?,
            None => Measure::Amount,
        };

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let current = plan::get(&mut conn, user_id, plan_id).await?;
        let quotas = checked_quotas(&current, &input.quotas)?;
        let habit_ids = self
            .checked_habits(&mut conn, user_id, &input.habit_ids)
            .await?;

        let name = input.name.and_then(optional_name);
        let position = plan::next_requirement_position(&mut conn, plan_id).await?;
        let valid_from = current.period.window(effective_day(on)).0;
        let id = plan::insert_requirement(
            &mut conn,
            NewRequirement {
                id: Uuid::new_v4(),
                plan_id,
                name: name.clone(),
                measure: measure.as_str().to_owned(),
                position,
                valid_from,
            },
            &habit_ids,
            &quotas,
        )
        .await?;

        Ok(Requirement {
            id,
            plan_id,
            name,
            quotas,
            measure,
            position,
            habit_ids,
            valid_from,
            valid_to: None,
        })
    }

    /// Different quotas, a different measure or a different set of habits change
    /// what a period asks for. When the version in force already covered earlier
    /// periods, it is closed and a new one opens with the current period, so the
    /// earlier periods keep their verdict. Anything else - a label, the order, or a
    /// version that only covers this period - is edited where it stands.
    pub async fn update_requirement(
        &self,
        id: Uuid,
        input: UpdateRequirement,
        on: Option<NaiveDate>,
    ) -> AppResult<()> {
        let measure = input.measure.as_deref().map(parse_measure).transpose()?;

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let plan_id = plan::requirement_plan(&mut conn, user_id, id).await?;
        let current_plan = plan::get(&mut conn, user_id, plan_id).await?;
        let current = plan::requirements_for(&mut conn, &[plan_id])
            .await?
            .into_iter()
            .find(|requirement| requirement.id == id)
            .ok_or(AppError::NotFound)?;
        let quotas = input
            .quotas
            .as_deref()
            .map(|quotas| checked_quotas(&current_plan, quotas))
            .transpose()?;
        let habit_ids = match input.habit_ids {
            Some(ids) => Some(self.checked_habits(&mut conn, user_id, &ids).await?),
            None => None,
        };
        let name = input.name.map(|value| value.and_then(optional_name));

        let reshapes = quotas
            .as_ref()
            .is_some_and(|quotas| *quotas != current.quotas)
            || measure.is_some_and(|measure| measure != current.measure)
            || habit_ids
                .as_deref()
                .is_some_and(|ids| !same_members(ids, &current.habit_ids));
        let start = current_plan.period.window(effective_day(on)).0;

        if reshapes && current.valid_from < start {
            let mut next = successor(&current, start);
            next.name = name.unwrap_or(current.name.clone());
            next.measure = measure.unwrap_or(current.measure).as_str().to_owned();
            next.position = input.position.unwrap_or(current.position);
            plan::replace_requirement(
                &mut conn,
                id,
                next,
                habit_ids.as_deref().unwrap_or(&current.habit_ids),
                quotas.as_deref().unwrap_or(&current.quotas),
            )
            .await?;
            return Ok(());
        }

        plan::update_requirement(
            &mut conn,
            id,
            RequirementChanges {
                name,
                measure: measure.map(|measure| measure.as_str().to_owned()),
                position: input.position,
            },
            habit_ids.as_deref(),
            quotas.as_deref(),
        )
        .await
    }

    /// Removes a requirement from the current period on. One that already covered
    /// earlier periods is closed rather than deleted, so those periods stay as they were.
    pub async fn delete_requirement(&self, id: Uuid, on: Option<NaiveDate>) -> AppResult<()> {
        let mut conn = self.conn().await?;
        let plan_id = plan::requirement_plan(&mut conn, self.user_id, id).await?;
        let current_plan = plan::get(&mut conn, self.user_id, plan_id).await?;
        let current = plan::requirements_for(&mut conn, &[plan_id])
            .await?
            .into_iter()
            .find(|requirement| requirement.id == id)
            .ok_or(AppError::NotFound)?;
        end(
            &mut conn,
            &current,
            current_plan.period.window(effective_day(on)).0,
        )
        .await
    }

    /// Rejects a requirement that names a habit the caller does not own.
    async fn checked_habits(
        &self,
        conn: &mut DbConn,
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

    /// How each plan fared over the last `count` periods, archived plans included
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

        let plans = plan::list(&mut conn, user_id, Some(period), true).await?;
        if plans.is_empty() {
            return Ok(Vec::new());
        }

        let windows: Vec<Window> = (0..count)
            .rev()
            .map(|back| period.window(period.step_back(on, back)))
            .collect();

        let plan_ids: Vec<Uuid> = plans.iter().map(|plan| plan.id).collect();
        let requirements = plan::requirements_for(&mut conn, &plan_ids).await?;
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
                plans: plans
                    .iter()
                    .filter_map(|plan| {
                        let asked = asked_of(plan, &requirements, window, &archived);
                        let climb = climb(plan, &asked, &bucket);
                        let base = climb.first()?.tier.id;
                        let deciding: Vec<&Requirement> = asked
                            .iter()
                            .filter(|requirement| requirement.quota_at(base).is_some())
                            .collect();
                        Some(PlanOutcome {
                            plan: plan.clone(),
                            reached: deciding
                                .iter()
                                .filter(|requirement| {
                                    requirement.quota_at(base).is_some_and(|quota| {
                                        done_for(requirement, &bucket) >= quota
                                    })
                                })
                                .count(),
                            total: deciding.len(),
                            met: met(&climb),
                            medals: medals(&climb),
                        })
                    })
                    .collect(),
            })
            .collect())
    }

    /// The plan screen: every plan of one period with what it asks now, how far
    /// along it is, the tiers it has climbed and its streak.
    pub async fn plan_progress(&self, period: Period, on: NaiveDate) -> AppResult<PeriodProgress> {
        let (period_start, period_end) = period.window(on);
        let user_id = self.user_id;
        let mut conn = self.conn().await?;

        let plans = plan::list(&mut conn, user_id, Some(period), false).await?;
        if plans.is_empty() {
            return Ok(PeriodProgress {
                period,
                period_start,
                period_end,
                plans: Vec::new(),
            });
        }

        let plan_ids: Vec<Uuid> = plans.iter().map(|plan| plan.id).collect();
        let requirements = plan::requirements_for(&mut conn, &plan_ids).await?;
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

        let asked: Vec<(Plan, Vec<Requirement>)> = plans
            .into_iter()
            .map(|plan| {
                let asked = asked_of(&plan, &requirements, now, &archived);
                (plan, asked)
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

        Ok(PeriodProgress {
            period,
            period_start,
            period_end,
            plans: asked
                .into_iter()
                .map(|(plan, asked)| PlanProgress {
                    streak: streak(&plan, &requirements, &windows, &buckets, &archived),
                    standing: climb(&plan, &asked, &current),
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
                            quotas: requirement.quotas.clone(),
                            measure: requirement.measure,
                            done: done_for(requirement, &current),
                        })
                        .collect(),
                    plan,
                })
                .collect(),
        })
    }
}

/// The next version of a requirement, opening at `start` with the same shape.
fn successor(requirement: &Requirement, start: NaiveDate) -> NewRequirement {
    NewRequirement {
        id: Uuid::new_v4(),
        plan_id: requirement.plan_id,
        name: requirement.name.clone(),
        measure: requirement.measure.as_str().to_owned(),
        position: requirement.position,
        valid_from: start,
    }
}

/// Stops a requirement from `start` on: closed when it covered earlier periods,
/// deleted when it never did.
async fn end(conn: &mut DbConn, requirement: &Requirement, start: NaiveDate) -> AppResult<()> {
    if requirement.valid_from < start {
        plan::close_requirement(conn, requirement.id, start).await
    } else {
        plan::delete_requirement(conn, requirement.id).await
    }
}

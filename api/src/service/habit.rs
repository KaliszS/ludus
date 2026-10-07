use chrono::Utc;
use uuid::Uuid;

use super::category::checked_category;
use super::{UserService, check_weekdays, parse_tracking, require_name};
use crate::domain::{Habit, Tracking};
use crate::error::{AppError, AppResult};
use crate::repo::habit::{self, HabitChanges, NewHabit};
use crate::repo::plan;

#[derive(Default)]
pub struct CreateHabit {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub unit: Option<String>,
    pub tracking: Option<String>,
    pub weekdays: Option<Vec<i16>>,
    pub category_id: Option<Uuid>,
}

/// Outer Option means "absent"; inner means "set to null".
#[derive(Default)]
pub struct UpdateHabit {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub unit: Option<Option<String>>,
    pub tracking: Option<String>,
    pub weekdays: Option<Option<Vec<i16>>>,
    pub category_id: Option<Option<Uuid>>,
    pub position: Option<f64>,
    pub archived: Option<bool>,
}

fn to_column(days: Vec<i16>) -> Option<Vec<Option<i16>>> {
    if days.is_empty() {
        None
    } else {
        Some(days.into_iter().map(Some).collect())
    }
}

impl UserService {
    pub async fn habits(&self, include_archived: bool) -> AppResult<Vec<Habit>> {
        let mut conn = self.conn().await?;
        habit::list(&mut conn, self.user_id, include_archived).await
    }

    pub async fn habit(&self, id: Uuid) -> AppResult<Habit> {
        let mut conn = self.conn().await?;
        habit::get(&mut conn, self.user_id, id).await
    }

    pub async fn create_habit(&self, input: CreateHabit) -> AppResult<Habit> {
        let name = require_name("name", &input.name)?;
        let tracking = match input.tracking.as_deref() {
            Some(value) => parse_tracking(value)?,
            None => Tracking::Binary,
        };
        let weekdays = input.weekdays.unwrap_or_default();
        check_weekdays(&weekdays)?;

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let position = habit::next_position(&mut conn, user_id).await?;
        let category_id = match input.category_id {
            Some(id) => Some(checked_category(&mut conn, user_id, id).await?),
            None => None,
        };

        habit::insert(
            &mut conn,
            NewHabit {
                id: Uuid::new_v4(),
                user_id,
                name,
                description: input.description.unwrap_or_default(),
                icon: input.icon,
                color: input.color,
                unit: input.unit,
                tracking: tracking.as_str().to_owned(),
                weekdays: to_column(weekdays),
                category_id,
                position,
            },
        )
        .await
    }

    pub async fn update_habit(&self, id: Uuid, input: UpdateHabit) -> AppResult<Habit> {
        let name = input
            .name
            .as_deref()
            .map(|v| require_name("name", v))
            .transpose()?;
        let tracking = input.tracking.as_deref().map(parse_tracking).transpose()?;
        if let Some(Some(days)) = input.weekdays.as_ref() {
            check_weekdays(days)?;
        }

        let mut conn = self.conn().await?;
        if let Some(Some(category_id)) = input.category_id {
            checked_category(&mut conn, self.user_id, category_id).await?;
        }
        // A binary check-in is stored as times = 1, which reads the same as a quantity
        // of one, so that direction keeps history intact. The reverse would have to
        // squash every recorded amount down to 1.
        if tracking == Some(Tracking::Binary)
            && habit::get(&mut conn, self.user_id, id).await?.tracking == Tracking::Quantity
        {
            return Err(AppError::invalid(
                "tracking",
                "a quantity habit cannot become binary without losing its amounts",
            ));
        }

        let changes = HabitChanges {
            name,
            description: input.description,
            icon: input.icon,
            color: input.color,
            unit: input.unit,
            tracking: tracking.map(|t| t.as_str().to_owned()),
            weekdays: input.weekdays.map(|days| days.and_then(to_column)),
            category_id: input.category_id,
            position: input.position,
            archived_at: input.archived.map(|on| on.then(Utc::now)),
        };
        habit::update(&mut conn, self.user_id, id, changes).await
    }

    pub async fn delete_habit(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        habit::delete(&mut conn, self.user_id, id).await?;
        plan::delete_orphaned_requirements(&mut conn, self.user_id).await?;
        Ok(())
    }
}

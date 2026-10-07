use uuid::Uuid;

use super::{UserService, require_name};
use crate::domain::HabitCategory;
use crate::error::{AppError, AppResult};
use crate::repo::category::{self, CategoryChanges, NewCategory};
use crate::repo::pool::DbConn;

pub struct CreateCategory {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// Outer Option means "absent"; inner means "set to null".
#[derive(Default)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
    pub position: Option<f64>,
}

/// Categories stop at two levels, so only a top-level one can take subcategories.
async fn checked_parent(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Uuid> {
    let parent = match category::get(conn, user_id, id).await {
        Err(AppError::NotFound) => {
            return Err(AppError::invalid("parent_id", "no such category"));
        }
        other => other?,
    };
    if parent.parent_id.is_some() {
        return Err(AppError::invalid(
            "parent_id",
            "a subcategory cannot hold subcategories",
        ));
    }
    Ok(parent.id)
}

/// What a habit may be filed under: any category of the same account.
pub async fn checked_category(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Uuid> {
    match category::get(conn, user_id, id).await {
        Err(AppError::NotFound) => Err(AppError::invalid("category_id", "no such category")),
        other => Ok(other?.id),
    }
}

impl UserService {
    pub async fn categories(&self) -> AppResult<Vec<HabitCategory>> {
        let mut conn = self.conn().await?;
        category::list(&mut conn, self.user_id).await
    }

    pub async fn create_category(&self, input: CreateCategory) -> AppResult<HabitCategory> {
        let name = require_name("name", &input.name)?;
        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let parent_id = match input.parent_id {
            Some(id) => Some(checked_parent(&mut conn, user_id, id).await?),
            None => None,
        };
        let position = category::next_position(&mut conn, user_id, parent_id).await?;

        category::insert(
            &mut conn,
            NewCategory {
                id: Uuid::new_v4(),
                user_id,
                parent_id,
                name,
                position,
            },
        )
        .await
    }

    /// Moving under another parent keeps the two-level limit both ways: the new parent
    /// must be top-level, and a category that has subcategories cannot become one.
    pub async fn update_category(
        &self,
        id: Uuid,
        input: UpdateCategory,
    ) -> AppResult<HabitCategory> {
        let name = input
            .name
            .as_deref()
            .map(|value| require_name("name", value))
            .transpose()?;

        let user_id = self.user_id;
        let mut conn = self.conn().await?;
        let current = category::get(&mut conn, user_id, id).await?;

        let mut position = input.position;
        if let Some(parent_id) = input.parent_id
            && parent_id != current.parent_id
        {
            if let Some(parent) = parent_id {
                if parent == id {
                    return Err(AppError::invalid(
                        "parent_id",
                        "a category cannot hold itself",
                    ));
                }
                checked_parent(&mut conn, user_id, parent).await?;
                if category::has_children(&mut conn, id).await? {
                    return Err(AppError::invalid(
                        "parent_id",
                        "a category with subcategories cannot become one",
                    ));
                }
            }
            if position.is_none() {
                position = Some(category::next_position(&mut conn, user_id, parent_id).await?);
            }
        }

        category::update(
            &mut conn,
            user_id,
            id,
            CategoryChanges {
                parent_id: input.parent_id,
                name,
                position,
            },
        )
        .await
    }

    pub async fn delete_category(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.conn().await?;
        category::delete(&mut conn, self.user_id, id).await
    }
}

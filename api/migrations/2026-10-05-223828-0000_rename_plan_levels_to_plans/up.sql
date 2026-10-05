-- A "level" was always a plan; now that plans get tiers, the two words must not
-- collide in the code. Renames only: no row or definition changes.
ALTER TABLE plan_levels RENAME TO plans;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_created_at_not_null TO plans_created_at_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_id_not_null TO plans_id_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_name_not_null TO plans_name_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_period_check TO plans_period_check;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_period_not_null TO plans_period_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_pkey TO plans_pkey;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_position_not_null TO plans_position_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_updated_at_not_null TO plans_updated_at_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_user_id_fkey TO plans_user_id_fkey;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_user_id_not_null TO plans_user_id_not_null;
ALTER TABLE plans RENAME CONSTRAINT plan_levels_user_id_period_name_key TO plans_user_id_period_name_key;
ALTER INDEX idx_plan_levels_user RENAME TO idx_plans_user;

ALTER TABLE plan_requirements RENAME COLUMN level_id TO plan_id;
ALTER TABLE plan_requirements RENAME CONSTRAINT plan_requirements_level_id_fkey TO plan_requirements_plan_id_fkey;
ALTER TABLE plan_requirements RENAME CONSTRAINT plan_requirements_level_id_not_null TO plan_requirements_plan_id_not_null;
ALTER TABLE plan_tasks RENAME COLUMN level_id TO plan_id;
ALTER TABLE plan_tasks RENAME CONSTRAINT plan_tasks_level_id_fkey TO plan_tasks_plan_id_fkey;
ALTER TABLE plan_tasks RENAME CONSTRAINT plan_tasks_level_id_not_null TO plan_tasks_plan_id_not_null;
ALTER INDEX idx_plan_requirements_level RENAME TO idx_plan_requirements_plan;

DROP TABLE plan_tasks;
DROP TABLE plan_quotas;
DROP TABLE plan_levels;
DROP TABLE task_goals;

ALTER TABLE tasks ADD COLUMN target_id uuid NULL REFERENCES habits(id) ON DELETE SET NULL;
CREATE INDEX idx_tasks_target ON tasks(target_id);
ALTER TABLE tasks ADD CONSTRAINT target_is_personal
    CHECK (target_id IS NULL OR team_id IS NULL);

DROP INDEX idx_projects_goal;
ALTER TABLE projects DROP COLUMN goal_id;
ALTER TABLE projects ADD COLUMN target_id uuid NULL REFERENCES habits(id) ON DELETE SET NULL;
CREATE INDEX idx_projects_target ON projects(target_id);

DROP INDEX idx_habits_goal;
ALTER TABLE habits DROP COLUMN goal_id;

DROP TABLE goals;

ALTER TABLE habits
    DROP COLUMN position,
    DROP COLUMN tracking,
    DROP COLUMN unit,
    DROP COLUMN color,
    DROP COLUMN icon,
    ADD COLUMN cadence_kind     text        NOT NULL DEFAULT 'none',
    ADD COLUMN cadence_period   text        NULL,
    ADD COLUMN times_per_period int         NULL CHECK (times_per_period IS NULL OR times_per_period > 0),
    ADD COLUMN completed_at     timestamptz NULL;

ALTER INDEX idx_habits_user RENAME TO idx_targets_user;
ALTER TABLE habit_checkins RENAME COLUMN habit_id TO target_id;
ALTER TABLE habit_checkins RENAME TO target_checkins;
ALTER TABLE habits RENAME TO targets;

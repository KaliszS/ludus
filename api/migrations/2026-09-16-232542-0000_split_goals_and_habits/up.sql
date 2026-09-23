ALTER TABLE targets RENAME TO habits;
ALTER TABLE target_checkins RENAME TO habit_checkins;
ALTER TABLE habit_checkins RENAME COLUMN target_id TO habit_id;
ALTER INDEX idx_targets_user RENAME TO idx_habits_user;

ALTER TABLE habits
    DROP COLUMN cadence_kind,
    DROP COLUMN cadence_period,
    DROP COLUMN times_per_period,
    DROP COLUMN completed_at,
    ADD COLUMN icon     text    NULL,
    ADD COLUMN color    text    NULL,
    ADD COLUMN unit     text    NULL,
    ADD COLUMN tracking text    NOT NULL DEFAULT 'binary',
    ADD COLUMN position float8  NOT NULL DEFAULT 0;

ALTER TABLE habits ALTER COLUMN position DROP DEFAULT;

-- Scope: neither set = personal, team_id = team, project_id = inside a project.
CREATE TABLE goals (
    id           uuid        PRIMARY KEY,
    user_id      uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    team_id      uuid        NULL REFERENCES teams(id) ON DELETE CASCADE,
    project_id   uuid        NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         text        NOT NULL,
    description  text        NOT NULL DEFAULT '',
    status       text        NOT NULL DEFAULT 'active',
    target_date  date        NULL,
    position     float8      NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL
);

CREATE INDEX idx_goals_user ON goals(user_id);
CREATE INDEX idx_goals_team ON goals(team_id);
CREATE INDEX idx_goals_project ON goals(project_id);

SELECT diesel_manage_updated_at('goals');

ALTER TABLE habits ADD COLUMN goal_id uuid NULL REFERENCES goals(id) ON DELETE SET NULL;
CREATE INDEX idx_habits_goal ON habits(goal_id);

DROP INDEX idx_projects_target;
ALTER TABLE projects DROP COLUMN target_id;
ALTER TABLE projects ADD COLUMN goal_id uuid NULL REFERENCES goals(id) ON DELETE SET NULL;
CREATE INDEX idx_projects_goal ON projects(goal_id);

-- The rule replacing target_is_personal (a team task must not link to a personal
-- goal) spans two rows, so it lives in service/ instead of as a CHECK.
ALTER TABLE tasks DROP CONSTRAINT target_is_personal;
DROP INDEX idx_tasks_target;
ALTER TABLE tasks DROP COLUMN target_id;

CREATE TABLE task_goals (
    task_id uuid NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    goal_id uuid NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, goal_id)
);

CREATE INDEX idx_task_goals_goal ON task_goals(goal_id);

CREATE TABLE plan_levels (
    id         uuid        PRIMARY KEY,
    user_id    uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       text        NOT NULL,
    period     text        NOT NULL CHECK (period IN ('day', 'week', 'month', 'quarter', 'year')),
    position   float8      NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, period, name)
);

CREATE INDEX idx_plan_levels_user ON plan_levels(user_id);

SELECT diesel_manage_updated_at('plan_levels');

-- A requirement is met by its member habits taken together, so one member is an
-- ordinary quota and several express "any N from this set".
CREATE TABLE plan_requirements (
    id       uuid   PRIMARY KEY,
    level_id uuid   NOT NULL REFERENCES plan_levels(id) ON DELETE CASCADE,
    name     text   NULL,
    quota    float8 NOT NULL CHECK (quota > 0),
    measure  text   NOT NULL DEFAULT 'amount' CHECK (measure IN ('amount', 'occurrences')),
    position float8 NOT NULL
);

CREATE INDEX idx_plan_requirements_level ON plan_requirements(level_id);

CREATE TABLE plan_requirement_habits (
    requirement_id uuid NOT NULL REFERENCES plan_requirements(id) ON DELETE CASCADE,
    habit_id       uuid NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    PRIMARY KEY (requirement_id, habit_id)
);

CREATE INDEX idx_plan_requirement_habits_habit ON plan_requirement_habits(habit_id);

-- period_start: a task commitment is one-off, so next period it is simply absent.
CREATE TABLE plan_tasks (
    level_id     uuid NOT NULL REFERENCES plan_levels(id) ON DELETE CASCADE,
    task_id      uuid NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    period_start date NOT NULL,
    PRIMARY KEY (level_id, task_id, period_start)
);

CREATE INDEX idx_plan_tasks_task ON plan_tasks(task_id);

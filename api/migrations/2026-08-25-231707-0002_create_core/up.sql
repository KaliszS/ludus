-- A NULL user_id marks a built-in row shared by the whole instance.
CREATE TABLE horizons (
    id         uuid        PRIMARY KEY,
    user_id    uuid        NULL REFERENCES users(id) ON DELETE CASCADE,
    name       text        NOT NULL,
    valid_days int         NULL CHECK (valid_days IS NULL OR valid_days > 0),
    position   float8      NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_horizons_user_name ON horizons(user_id, name) WHERE user_id IS NOT NULL;
CREATE UNIQUE INDEX idx_horizons_builtin_name ON horizons(name) WHERE user_id IS NULL;

SELECT diesel_manage_updated_at('horizons');

INSERT INTO horizons (id, user_id, name, valid_days, position) VALUES
    (gen_random_uuid(), NULL, 'Today', 1,    100),
    (gen_random_uuid(), NULL, 'Soon',  30,   200),
    (gen_random_uuid(), NULL, 'Later', NULL, 300);

CREATE TABLE workflow_states (
    id         uuid        PRIMARY KEY,
    team_id    uuid        NULL REFERENCES teams(id) ON DELETE CASCADE,
    name       text        NOT NULL,
    category   text        NOT NULL CHECK (category IN ('backlog', 'unstarted', 'started', 'completed', 'canceled')),
    color      text        NOT NULL DEFAULT '#8b8b8b',
    position   float8      NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_states_team ON workflow_states(team_id);

SELECT diesel_manage_updated_at('workflow_states');

INSERT INTO workflow_states (id, team_id, name, category, position) VALUES
    (gen_random_uuid(), NULL, 'Backlog',     'backlog',   100),
    (gen_random_uuid(), NULL, 'Todo',        'unstarted', 200),
    (gen_random_uuid(), NULL, 'In Progress', 'started',   300),
    (gen_random_uuid(), NULL, 'Done',        'completed', 400),
    (gen_random_uuid(), NULL, 'Canceled',    'canceled',  500);

CREATE TABLE targets (
    id               uuid        PRIMARY KEY,
    user_id          uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name             text        NOT NULL,
    description      text        NOT NULL DEFAULT '',
    cadence_kind     text        NOT NULL DEFAULT 'none',
    cadence_period   text        NULL,
    times_per_period int         NULL CHECK (times_per_period IS NULL OR times_per_period > 0),
    weekdays         smallint[]  NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    completed_at     timestamptz NULL,
    archived_at      timestamptz NULL
);

CREATE INDEX idx_targets_user ON targets(user_id) WHERE archived_at IS NULL;

SELECT diesel_manage_updated_at('targets');

CREATE TABLE target_checkins (
    target_id  uuid        NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    day        date        NOT NULL,
    times      int         NOT NULL DEFAULT 1 CHECK (times > 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (target_id, day)
);

SELECT diesel_manage_updated_at('target_checkins');

CREATE TABLE projects (
    id           uuid        PRIMARY KEY,
    team_id      uuid        NULL REFERENCES teams(id) ON DELETE CASCADE,
    creator_id   uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         text        NOT NULL,
    description  text        NOT NULL DEFAULT '',
    status       text        NOT NULL DEFAULT 'planned',
    target_id    uuid        NULL REFERENCES targets(id) ON DELETE SET NULL,
    target_date  date        NULL,
    position     float8      NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL
);

CREATE INDEX idx_projects_team ON projects(team_id);
CREATE INDEX idx_projects_creator ON projects(creator_id);
CREATE INDEX idx_projects_target ON projects(target_id);

SELECT diesel_manage_updated_at('projects');

CREATE TABLE project_members (
    project_id uuid        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id    uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role       text        NOT NULL DEFAULT 'member',
    added_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id)
);

CREATE INDEX idx_project_members_user ON project_members(user_id);

CREATE TABLE milestones (
    id           uuid        PRIMARY KEY,
    project_id   uuid        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         text        NOT NULL,
    description  text        NOT NULL DEFAULT '',
    target_date  date        NULL,
    position     float8      NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL
);

CREATE INDEX idx_milestones_project ON milestones(project_id);

SELECT diesel_manage_updated_at('milestones');

CREATE TABLE milestone_members (
    milestone_id uuid        NOT NULL REFERENCES milestones(id) ON DELETE CASCADE,
    user_id      uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role         text        NOT NULL DEFAULT 'member',
    added_at     timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (milestone_id, user_id)
);

CREATE INDEX idx_milestone_members_user ON milestone_members(user_id);

CREATE TABLE cycles (
    id         uuid        PRIMARY KEY,
    team_id    uuid        NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    seq        int         NOT NULL,
    name       text        NOT NULL DEFAULT '',
    starts_on  date        NOT NULL,
    ends_on    date        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (team_id, seq),
    CHECK (ends_on >= starts_on)
);

SELECT diesel_manage_updated_at('cycles');

CREATE TABLE tasks (
    id                uuid        PRIMARY KEY,
    team_id           uuid        NULL REFERENCES teams(id) ON DELETE CASCADE,
    seq               int         NULL,
    creator_id        uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assignee_id       uuid        NULL REFERENCES users(id) ON DELETE SET NULL,
    title             text        NOT NULL,
    description       text        NOT NULL DEFAULT '',
    state_id          uuid        NOT NULL REFERENCES workflow_states(id),
    priority          smallint    NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 4),
    estimate          float8      NULL,
    parent_id         uuid        NULL REFERENCES tasks(id) ON DELETE SET NULL,
    project_id        uuid        NULL REFERENCES projects(id) ON DELETE SET NULL,
    milestone_id      uuid        NULL REFERENCES milestones(id) ON DELETE SET NULL,
    cycle_id          uuid        NULL REFERENCES cycles(id) ON DELETE SET NULL,
    target_id         uuid        NULL REFERENCES targets(id) ON DELETE SET NULL,
    is_milestone_goal boolean     NOT NULL DEFAULT false,
    horizon_id        uuid        NULL REFERENCES horizons(id) ON DELETE SET NULL,
    horizon_set_at    timestamptz NULL,
    due_date          date        NULL,
    position          float8      NOT NULL,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    completed_at      timestamptz NULL,
    canceled_at       timestamptz NULL,

    -- ENG-42 only exists inside a team, so seq and team_id come and go together.
    CONSTRAINT seq_belongs_to_team CHECK ((team_id IS NULL) = (seq IS NULL)),
    -- A target is a private life goal, so only a personal task may point at one.
    CONSTRAINT target_is_personal CHECK (target_id IS NULL OR team_id IS NULL),
    CONSTRAINT milestone_goal_needs_milestone
        CHECK (is_milestone_goal = false OR milestone_id IS NOT NULL),
    CONSTRAINT horizon_needs_timestamp
        CHECK (horizon_id IS NULL OR horizon_set_at IS NOT NULL)
);

CREATE UNIQUE INDEX idx_tasks_team_seq ON tasks(team_id, seq) WHERE team_id IS NOT NULL;
CREATE UNIQUE INDEX idx_tasks_milestone_goal ON tasks(milestone_id) WHERE is_milestone_goal;
CREATE INDEX idx_tasks_team ON tasks(team_id);
CREATE INDEX idx_tasks_creator ON tasks(creator_id);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_state ON tasks(state_id);
CREATE INDEX idx_tasks_parent ON tasks(parent_id);
CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_milestone ON tasks(milestone_id);
CREATE INDEX idx_tasks_cycle ON tasks(cycle_id);
CREATE INDEX idx_tasks_target ON tasks(target_id);
CREATE INDEX idx_tasks_horizon ON tasks(horizon_id);

SELECT diesel_manage_updated_at('tasks');

CREATE TABLE labels (
    id         uuid        PRIMARY KEY,
    team_id    uuid        NULL REFERENCES teams(id) ON DELETE CASCADE,
    name       text        NOT NULL,
    color      text        NOT NULL DEFAULT '#8b8b8b',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_labels_team_name ON labels(team_id, name) WHERE team_id IS NOT NULL;
CREATE UNIQUE INDEX idx_labels_global_name ON labels(name) WHERE team_id IS NULL;

SELECT diesel_manage_updated_at('labels');

CREATE TABLE task_labels (
    task_id  uuid NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    label_id uuid NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, label_id)
);

CREATE INDEX idx_task_labels_label ON task_labels(label_id);

CREATE TABLE comments (
    id         uuid        PRIMARY KEY,
    task_id    uuid        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    author_id  uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id  uuid        NULL REFERENCES comments(id) ON DELETE CASCADE,
    body       text        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    edited_at  timestamptz NULL
);

CREATE INDEX idx_comments_task ON comments(task_id);
CREATE INDEX idx_comments_parent ON comments(parent_id);

SELECT diesel_manage_updated_at('comments');

CREATE TABLE task_relations (
    from_task_id uuid        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    to_task_id   uuid        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    kind         text        NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (from_task_id, to_task_id, kind),
    CHECK (from_task_id <> to_task_id)
);

CREATE INDEX idx_task_relations_to ON task_relations(to_task_id);

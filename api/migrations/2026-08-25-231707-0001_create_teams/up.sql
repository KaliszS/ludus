-- next_seq is the counter behind ENG-42; bump it with UPDATE ... RETURNING.
CREATE TABLE teams (
    id         uuid        PRIMARY KEY,
    key        text        NOT NULL UNIQUE,
    name       text        NOT NULL,
    next_seq   int         NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

SELECT diesel_manage_updated_at('teams');

CREATE TABLE team_members (
    team_id   uuid        NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id   uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role      text        NOT NULL DEFAULT 'member',
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (team_id, user_id)
);

CREATE INDEX idx_team_members_user ON team_members(user_id);

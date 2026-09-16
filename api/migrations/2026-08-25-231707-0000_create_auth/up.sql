CREATE TABLE users (
    id           uuid        PRIMARY KEY,
    google_sub   text        NOT NULL UNIQUE,
    email        text        NOT NULL,
    display_name text        NOT NULL,
    avatar_url   text        NULL,
    status       text        NOT NULL DEFAULT 'pending',
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

SELECT diesel_manage_updated_at('users');

-- A token matching prev_hash is a replayed one: delete the row, kill the session.
CREATE TABLE sessions (
    id           uuid        PRIMARY KEY,
    user_id      uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash   bytea       NOT NULL UNIQUE,
    prev_hash    bytea       NULL,
    client       text        NOT NULL,
    user_agent   text        NULL,
    expires_at   timestamptz NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_prev_hash ON sessions(prev_hash) WHERE prev_hash IS NOT NULL;
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

CREATE TABLE device_authorizations (
    id               uuid        PRIMARY KEY,
    device_code_hash bytea       NOT NULL UNIQUE,
    user_code        text        NOT NULL UNIQUE,
    user_id          uuid        NULL REFERENCES users(id) ON DELETE CASCADE,
    status           text        NOT NULL DEFAULT 'pending',
    expires_at       timestamptz NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    last_polled_at   timestamptz NULL
);

CREATE INDEX idx_device_authorizations_expires ON device_authorizations(expires_at);

CREATE TABLE oauth_states (
    state            text        PRIMARY KEY,
    pkce_verifier    text        NOT NULL,
    redirect_uri     text        NOT NULL,
    device_code_hash bytea       NULL,
    expires_at       timestamptz NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_oauth_states_expires ON oauth_states(expires_at);

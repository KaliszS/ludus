-- A sign-in now has two legs: the provider returns to the daemon, then the client
-- trades a one-time code for a session. The row carries the client's PKCE challenge
-- from the first leg and the resolved user into the second. Rows live ten minutes,
-- so existing ones are simply dropped.
DELETE FROM oauth_states;

ALTER TABLE oauth_states
    ADD COLUMN provider         text  NOT NULL CHECK (provider IN ('google', 'github')),
    ADD COLUMN client_challenge text  NULL,
    ADD COLUMN user_id          uuid  NULL REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN code_hash        bytea NULL UNIQUE;

CREATE INDEX idx_oauth_states_user ON oauth_states(user_id);

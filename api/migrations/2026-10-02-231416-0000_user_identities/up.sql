-- One person, many logins. Keeping the provider's subject on users would mean a
-- data migration the day a second provider lands; here it is just another row.
CREATE TABLE user_identities (
    id          uuid        PRIMARY KEY,
    user_id     uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider    text        NOT NULL CHECK (provider IN ('google', 'github')),
    provider_id text        NOT NULL,
    email       text        NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_id)
);

CREATE INDEX idx_user_identities_user ON user_identities(user_id);

INSERT INTO user_identities (id, user_id, provider, provider_id, email)
SELECT gen_random_uuid(), id, 'google', google_sub, email FROM users;

ALTER TABLE users DROP COLUMN google_sub;

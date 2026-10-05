DROP INDEX idx_oauth_states_user;

ALTER TABLE oauth_states
    DROP COLUMN code_hash,
    DROP COLUMN user_id,
    DROP COLUMN client_challenge,
    DROP COLUMN provider;

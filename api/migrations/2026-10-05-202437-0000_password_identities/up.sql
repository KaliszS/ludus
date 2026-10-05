-- A password is one more way into an account, next to the providers, so it lives
-- with them: provider 'password', provider_id the lowercased email it signs in with.
-- The pairing check keeps a hash from ever sitting on a provider row, and a
-- password row from ever lacking one.
ALTER TABLE user_identities ADD COLUMN password_hash text NULL;

ALTER TABLE user_identities DROP CONSTRAINT user_identities_provider_check;
ALTER TABLE user_identities
    ADD CONSTRAINT user_identities_provider_check
        CHECK (provider IN ('google', 'github', 'password'));

ALTER TABLE user_identities
    ADD CONSTRAINT user_identities_password_hash_check
        CHECK ((provider = 'password') = (password_hash IS NOT NULL));

ALTER TABLE users ADD COLUMN google_sub text;

UPDATE users u SET google_sub = i.provider_id
FROM user_identities i WHERE i.user_id = u.id AND i.provider = 'google';

DELETE FROM users WHERE google_sub IS NULL;
ALTER TABLE users ALTER COLUMN google_sub SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_google_sub_key UNIQUE (google_sub);

DROP TABLE user_identities;

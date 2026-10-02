DROP INDEX idx_device_authorizations_user;
DROP INDEX idx_comments_author;

DROP INDEX idx_habits_user;
CREATE INDEX idx_habits_user ON habits(user_id) WHERE archived_at IS NULL;

ALTER TABLE habit_checkins DROP CONSTRAINT habit_checkins_pkey;
ALTER TABLE habit_checkins ADD CONSTRAINT target_checkins_pkey PRIMARY KEY (habit_id, day);

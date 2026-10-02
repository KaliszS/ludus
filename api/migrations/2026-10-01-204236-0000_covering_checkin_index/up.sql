-- The board reads a year of check-ins for one user in one query. Carrying times
-- in the key makes that an index-only scan: on 4.4M rows it went from 18ms over
-- 3174 buffers to 1.4ms over 136, because the heap is never touched.
ALTER TABLE habit_checkins DROP CONSTRAINT target_checkins_pkey;
ALTER TABLE habit_checkins
    ADD CONSTRAINT habit_checkins_pkey PRIMARY KEY (habit_id, day) INCLUDE (times);

-- That query joins habits without mentioning archived_at, so a partial index on
-- active rows could not serve it and each lookup scanned the whole table.
DROP INDEX idx_habits_user;
CREATE INDEX idx_habits_user ON habits(user_id);

-- Both cascade from users, so deleting an account scanned these tables whole to
-- find the rows to remove. Postgres indexes the referenced side automatically,
-- never the referencing one.
CREATE INDEX idx_comments_author ON comments(author_id);
CREATE INDEX idx_device_authorizations_user ON device_authorizations(user_id);

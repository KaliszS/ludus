-- Left over from the table rename in split_goals_and_habits. Postgres keeps
-- constraint names through ALTER TABLE ... RENAME, so every one of these still
-- said 'target'. Nothing in the code refers to them; this is for reading the schema.
ALTER TABLE habits RENAME CONSTRAINT targets_pkey TO habits_pkey;
ALTER TABLE habits RENAME CONSTRAINT targets_user_id_fkey TO habits_user_id_fkey;
ALTER TABLE habits RENAME CONSTRAINT targets_id_not_null TO habits_id_not_null;
ALTER TABLE habits RENAME CONSTRAINT targets_user_id_not_null TO habits_user_id_not_null;
ALTER TABLE habits RENAME CONSTRAINT targets_name_not_null TO habits_name_not_null;
ALTER TABLE habits RENAME CONSTRAINT targets_description_not_null TO habits_description_not_null;
ALTER TABLE habits RENAME CONSTRAINT targets_created_at_not_null TO habits_created_at_not_null;
ALTER TABLE habits RENAME CONSTRAINT targets_updated_at_not_null TO habits_updated_at_not_null;

ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_times_check TO habit_checkins_times_check;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_target_id_fkey TO habit_checkins_habit_id_fkey;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_target_id_not_null TO habit_checkins_habit_id_not_null;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_day_not_null TO habit_checkins_day_not_null;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_times_not_null TO habit_checkins_times_not_null;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_created_at_not_null TO habit_checkins_created_at_not_null;
ALTER TABLE habit_checkins RENAME CONSTRAINT target_checkins_updated_at_not_null TO habit_checkins_updated_at_not_null;

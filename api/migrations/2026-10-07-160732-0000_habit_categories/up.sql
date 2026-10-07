-- Habits sort into categories at most two levels deep, e.g. "Sport" holding "Running"
-- and "Strength". The depth limit spans rows, so it lives in service/. Removing a
-- category takes its subcategories along and leaves their habits uncategorized.
CREATE TABLE habit_categories (
    id         uuid        PRIMARY KEY,
    user_id    uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id  uuid        NULL REFERENCES habit_categories(id) ON DELETE CASCADE,
    name       text        NOT NULL,
    position   float8      NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT habit_categories_name_key UNIQUE NULLS NOT DISTINCT (user_id, parent_id, name)
);

CREATE INDEX idx_habit_categories_parent ON habit_categories(parent_id);

SELECT diesel_manage_updated_at('habit_categories');

ALTER TABLE habits
    ADD COLUMN category_id uuid NULL REFERENCES habit_categories(id) ON DELETE SET NULL;
CREATE INDEX idx_habits_category ON habits(category_id);

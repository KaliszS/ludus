-- Plans used to be judged by today's definition all the way back, so any edit
-- rewrote history. A requirement row is now one version, in force from valid_from
-- up to valid_to (exclusive), both on its level's period boundaries. A change that
-- reaches into the past closes the version and opens the next one.
ALTER TABLE plan_requirements
    ADD COLUMN valid_from date NULL,
    ADD COLUMN valid_to   date NULL;

-- Existing requirements count from the period their level was created in.
UPDATE plan_requirements r
SET valid_from = CASE l.period
        WHEN 'day' THEN l.created_at::date
        ELSE date_trunc(l.period, l.created_at)::date
    END
FROM plan_levels l
WHERE l.id = r.level_id;

ALTER TABLE plan_requirements ALTER COLUMN valid_from SET NOT NULL;
ALTER TABLE plan_requirements
    ADD CONSTRAINT plan_requirements_validity_check
        CHECK (valid_to IS NULL OR valid_to > valid_from);

-- An archived level stops counting from the period it was archived in and keeps
-- every period before that.
ALTER TABLE plan_levels ADD COLUMN archived_on date NULL;

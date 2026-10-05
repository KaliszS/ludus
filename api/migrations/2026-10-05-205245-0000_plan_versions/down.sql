-- Without versions only the current definition can survive.
DELETE FROM plan_requirements WHERE valid_to IS NOT NULL;

ALTER TABLE plan_levels DROP COLUMN archived_on;
ALTER TABLE plan_requirements DROP CONSTRAINT plan_requirements_validity_check;
ALTER TABLE plan_requirements DROP COLUMN valid_to, DROP COLUMN valid_from;

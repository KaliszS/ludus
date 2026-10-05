ALTER TABLE plan_requirements ADD COLUMN quota float8 NULL;

-- Without tiers, each version keeps the quota of its lowest tier.
UPDATE plan_requirements r
SET quota = lowest.quota
FROM (
    SELECT DISTINCT ON (q.requirement_id) q.requirement_id, q.quota
    FROM plan_requirement_quotas q
    JOIN plan_tiers t ON t.id = q.tier_id
    ORDER BY q.requirement_id, t.position
) lowest
WHERE lowest.requirement_id = r.id;

ALTER TABLE plan_requirements ALTER COLUMN quota SET NOT NULL;
ALTER TABLE plan_requirements ADD CONSTRAINT plan_requirements_quota_check CHECK (quota > 0);

DROP TABLE plan_requirement_quotas;
DROP TABLE plan_tiers;

-- A plan can be reached at several tiers - minimum, optimum, maximum, or just the
-- medals - and a requirement version gives a quota per tier. A missing quota row is
-- a tier that goal skips. Tiers are retired, never deleted: past versions still
-- point at them, and their quotas are how those periods are judged.
CREATE TABLE plan_tiers (
    id         uuid        PRIMARY KEY,
    plan_id    uuid        NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    name       text        NULL,
    medal      text        NULL CHECK (medal IN ('bronze', 'silver', 'gold')),
    position   float8      NOT NULL,
    retired_on date        NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_plan_tiers_plan ON plan_tiers(plan_id);

CREATE TABLE plan_requirement_quotas (
    requirement_id uuid   NOT NULL REFERENCES plan_requirements(id) ON DELETE CASCADE,
    tier_id        uuid   NOT NULL REFERENCES plan_tiers(id) ON DELETE CASCADE,
    quota          float8 NOT NULL CHECK (quota > 0),
    PRIMARY KEY (requirement_id, tier_id)
);

CREATE INDEX idx_plan_requirement_quotas_tier ON plan_requirement_quotas(tier_id);

-- Every existing plan becomes one tier without a medal: exactly what it was.
INSERT INTO plan_tiers (id, plan_id, position)
SELECT gen_random_uuid(), id, 100 FROM plans;

INSERT INTO plan_requirement_quotas (requirement_id, tier_id, quota)
SELECT r.id, t.id, r.quota
FROM plan_requirements r
JOIN plan_tiers t ON t.plan_id = r.plan_id;

ALTER TABLE plan_requirements DROP COLUMN quota;

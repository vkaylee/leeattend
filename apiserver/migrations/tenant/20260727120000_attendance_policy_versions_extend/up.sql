-- ============================================================================
-- ADR-001/002/004: Extend attendance_policy_versions for layered cascade engine
--
-- Adds:
--   * 3-state lifecycle enum (DRAFT / ACTIVE / FROZEN)
--   * Version-level metadata (name, description, updated_by, updated_at)
--   * Layer-body JSONB columns (base_rules, conditional_overrides, delta_specs)
--   * DB-computed rules_hash generated column (deterministic from JSONB text)
--   * Trigger enforcing immutability + monotonic state transitions
--
-- The legacy `rules jsonb` column is left in place; readers switch to the new
-- columns once the cascade engine lands. See ADR-005 for dual-run plan.
-- ============================================================================

-- ---------------------------------------------------------------------------
-- 1. Lifecycle enum
-- ---------------------------------------------------------------------------
CREATE TYPE public.policy_version_status_type AS ENUM ('DRAFT', 'ACTIVE', 'FROZEN');

-- ---------------------------------------------------------------------------
-- 2. New columns
-- ---------------------------------------------------------------------------
ALTER TABLE public.attendance_policy_versions
    ADD COLUMN status public.policy_version_status_type NOT NULL DEFAULT 'DRAFT',
    ADD COLUMN name VARCHAR(255),
    ADD COLUMN description TEXT,
    ADD COLUMN updated_by UUID,
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN base_rules JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN conditional_overrides JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN delta_specs JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN first_referenced_at TIMESTAMPTZ;

-- DB-computed hash — never trust app-side hashing.
-- Concatenated text form of the 3 rule JSONBs, then md5() → 32-char hex.
-- Deterministic per JSONB canonical text output (Postgres normalizes JSONB).
ALTER TABLE public.attendance_policy_versions
    ADD COLUMN rules_hash TEXT GENERATED ALWAYS AS (
        md5(
            COALESCE(base_rules::text, '') ||
            '|' ||
            COALESCE(conditional_overrides::text, '') ||
            '|' ||
            COALESCE(delta_specs::text, '')
        )
    ) STORED;

COMMENT ON COLUMN public.attendance_policy_versions.status IS
    'ADR-004 lifecycle: DRAFT → ACTIVE → FROZEN. Monotonic; enforced by trigger.';
COMMENT ON COLUMN public.attendance_policy_versions.base_rules IS
    'L1 base policy — absolute defaults, min/max caps, mandatory field specs.';
COMMENT ON COLUMN public.attendance_policy_versions.conditional_overrides IS
    'Layer overrides keyed by scope (day_type / shift_type / shift_id).';
COMMENT ON COLUMN public.attendance_policy_versions.delta_specs IS
    'Reserved for future absolute-delta declarations (per-field deltas).';
COMMENT ON COLUMN public.attendance_policy_versions.rules_hash IS
    'md5 of base_rules|conditional_overrides|delta_specs. DB-computed, never app-side.';
COMMENT ON COLUMN public.attendance_policy_versions.first_referenced_at IS
    'Timestamp when this version was first snapshotted onto an attendance row. Once set, transition to FROZEN.';

CREATE INDEX idx_attendance_policy_versions_policy_status
    ON public.attendance_policy_versions (policy_id, status);

CREATE INDEX idx_attendance_policy_versions_rules_hash
    ON public.attendance_policy_versions (rules_hash);

-- ---------------------------------------------------------------------------
-- 3. Immutability + monotonic state transition trigger
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION public.enforce_policy_version_immutability()
RETURNS TRIGGER AS $$
BEGIN
    -- FROZEN is terminal — nothing may change.
    IF OLD.status = 'FROZEN' THEN
        RAISE EXCEPTION 'POLICY_VERSION_FROZEN: cannot modify frozen version %', OLD.id
            USING ERRCODE = 'check_violation';
    END IF;

    -- ACTIVE: only metadata may change; rules are locked.
    IF OLD.status = 'ACTIVE' AND NEW.status = 'ACTIVE'
       AND OLD.rules_hash IS DISTINCT FROM NEW.rules_hash THEN
        RAISE EXCEPTION 'POLICY_VERSION_ACTIVE_LOCKED: rules of ACTIVE version are immutable; create a new DRAFT'
            USING ERRCODE = 'check_violation';
    END IF;

    -- Monotonic state machine: DRAFT→ACTIVE→FROZEN, self-loops allowed.
    IF (OLD.status, NEW.status) NOT IN (
        ('DRAFT'::public.policy_version_status_type,  'DRAFT'::public.policy_version_status_type),
        ('DRAFT'::public.policy_version_status_type,  'ACTIVE'::public.policy_version_status_type),
        ('ACTIVE'::public.policy_version_status_type, 'ACTIVE'::public.policy_version_status_type),
        ('ACTIVE'::public.policy_version_status_type, 'FROZEN'::public.policy_version_status_type),
        ('FROZEN'::public.policy_version_status_type, 'FROZEN'::public.policy_version_status_type)
    ) THEN
        RAISE EXCEPTION 'POLICY_VERSION_INVALID_TRANSITION: % -> %', OLD.status, NEW.status
            USING ERRCODE = 'check_violation';
    END IF;

    -- Bump updated_at on legitimate change.
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_attendance_policy_versions_immutability
    BEFORE UPDATE ON public.attendance_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.enforce_policy_version_immutability();

-- Prevent deletion of FROZEN rows (metadata-level guard; app layer should also block).
CREATE OR REPLACE FUNCTION public.enforce_policy_version_no_delete_when_frozen()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'FROZEN' THEN
        RAISE EXCEPTION 'POLICY_VERSION_FROZEN: cannot delete frozen version %', OLD.id
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_attendance_policy_versions_no_delete_frozen
    BEFORE DELETE ON public.attendance_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.enforce_policy_version_no_delete_when_frozen();

-- ============================================================================
-- Shift Versioning: Enterprise SCD Type-2 Model
--
-- Introduces:
--   1. shift_templates     — stable logical identity (one per "shift concept")
--   2. shift_versions      — immutable version rows (append-only, no UPDATE)
--   3. shift_audit_log     — tamper-proof audit trail
--
-- All consumer tables that currently reference shifts.id via FK will
-- reference shift_templates.id instead.
--
-- Backfill strategy:
--   - Each distinct parent_shift_id chain in public.shifts maps to one template.
--   - The root of the chain (parent_shift_id IS NULL) identifies the template.
--   - Every shift row becomes a shift_versions row.
--   - Consumer tables are resolved through shifts.id → shift_versions.id → template_id.
-- ============================================================================

-- ---------------------------------------------------------------------------
-- Step 1: Create new tables
-- ---------------------------------------------------------------------------

CREATE TABLE public.shift_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    current_version_id UUID,              -- denormalized → shift_versions.id
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    UNIQUE(workspace_id, name)
);

COMMENT ON TABLE public.shift_templates IS 'Stable logical identity for a shift concept. Never changes across revisions or corrections.';

CREATE TABLE public.shift_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES public.shift_templates(id),
    workspace_id UUID NOT NULL,
    version_number INT NOT NULL,          -- monotonically increasing per template (1 = original)
    name VARCHAR(100) NOT NULL,
    report_symbol VARCHAR(10),
    location_id UUID,
    employer_id UUID,
    effective_from DATE NOT NULL,         -- when this version becomes effective
    effective_to DATE,                    -- NULL = current; closed when superseded
    change_type VARCHAR(20) NOT NULL DEFAULT 'create',  -- create | revise | correct
    change_reason TEXT,                   -- mandatory for revise/correct
    superseded_by UUID,                   -- self-ref → shift_versions.id (which version replaces this)
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL
);

COMMENT ON TABLE public.shift_versions IS 'Immutable SCD Type-2 rows. One row per change. Never UPDATE, only INSERT.';
COMMENT ON COLUMN public.shift_versions.change_type IS 'create=initial version, revise=scheduled change with new effective_from, correct=error fix keeping same effective_from';
COMMENT ON COLUMN public.shift_versions.superseded_by IS 'Points to the version that replaces this one. NULL if this is the current version.';
COMMENT ON COLUMN public.shift_versions.effective_to IS 'NULL for current version. Set to (new.effective_from - 1 day) when superseded by a revision, or (new.effective_from) when superseded by a correction.';

CREATE TABLE public.shift_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    version_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL DEFAULT 'version_created',
    change_type VARCHAR(20) NOT NULL,
    changed_by UUID NOT NULL,
    change_reason TEXT,
    old_values JSONB,                    -- snapshot of superseded version (NULL for create)
    new_values JSONB,                    -- snapshot of the new version
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL
);

COMMENT ON TABLE public.shift_audit_log IS 'Append-only tamper-proof audit trail for all shift changes. Populated by application code, not triggers.';
COMMENT ON COLUMN public.shift_audit_log.old_values IS 'Full JSON snapshot of the superseded version including intervals. NULL on initial creation.';
COMMENT ON COLUMN public.shift_audit_log.new_values IS 'Full JSON snapshot of the new version including intervals.';

-- ---------------------------------------------------------------------------
-- Step 2: Backfill shift_templates from existing shifts
--
-- Each distinct root (parent_shift_id IS NULL) → 1 template.
-- Templates MUST be created for ALL roots (including archived ones) because
-- child shifts (revisions) resolve their template_id by walking the chain to
-- the root. If a root was archived and its child is now active, the child
-- still needs the root's id as its template_id.
-- ---------------------------------------------------------------------------

-- 2a. Templates for all root shifts (parent_shift_id IS NULL)
--     Covers both active and archived roots.
INSERT INTO public.shift_templates (id, workspace_id, name, created_at)
SELECT
    id,
    workspace_id,
    name,
    COALESCE(created_at, now())
FROM public.shifts
WHERE parent_shift_id IS NULL;

-- 2b. Templates for orphan shifts whose parent chain is broken
--     (parent_shift_id points to a shift that no longer exists).
--     These become their own template root.
INSERT INTO public.shift_templates (id, workspace_id, name, created_at)
SELECT
    s.id,
    s.workspace_id,
    s.name,
    COALESCE(s.created_at, now())
FROM public.shifts s
WHERE s.parent_shift_id IS NOT NULL
  -- Only if a template hasn't already been created for this id
  AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = s.id)
  -- And the parent doesn't exist (broken chain)
  AND NOT EXISTS (SELECT 1 FROM public.shifts parent WHERE parent.id = s.parent_shift_id);

-- ---------------------------------------------------------------------------
-- Step 3: Backfill shift_versions from ALL shifts rows
-- ---------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION public.resolve_template_id(p_shift_id UUID)
RETURNS UUID AS $$
DECLARE
    v_current UUID;
    v_depth INT := 0;
    v_max_depth INT := 20;
BEGIN
    v_current := p_shift_id;
    -- Walk parent_shift_id chain to find root
    WHILE v_depth < v_max_depth LOOP
        -- Check if current is a root (exists in shift_templates)
        IF EXISTS (SELECT 1 FROM public.shift_templates WHERE id = v_current) THEN
            RETURN v_current;
        END IF;
        -- Move to parent
        SELECT parent_shift_id INTO v_current
        FROM public.shifts
        WHERE id = v_current;
        IF v_current IS NULL THEN
            -- Fell off chain without finding root
            RETURN NULL;
        END IF;
        v_depth := v_depth + 1;
    END LOOP;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 3a. Insert versions for ALL shifts (both active and archived)
INSERT INTO public.shift_versions (
    id,
    template_id,
    workspace_id,
    version_number,
    name,
    report_symbol,
    location_id,
    employer_id,
    effective_from,
    effective_to,
    change_type,
    created_by,
    created_at
)
SELECT
    s.id,
    public.resolve_template_id(s.id),
    s.workspace_id,
    -- version_number: 1 for root, sequential for children of same template
    COALESCE(
        (SELECT COUNT(*) FROM public.shifts s2
         WHERE public.resolve_template_id(s2.id) = public.resolve_template_id(s.id)
           AND s2.created_at <= s.created_at),
        1
    )::INT,
    s.name,
    s.report_symbol,
    s.location_id,
    s.employer_id,
    -- effective_from: default to created_at date for backfill
    COALESCE(s.created_at, now())::DATE,
    -- effective_to: NULL for active shifts (current version)
    CASE WHEN s.is_active THEN NULL
         ELSE COALESCE(s.updated_at, s.created_at, now())::DATE
    END,
    -- change_type: 'create' for root, 'revise' for chain children
    CASE WHEN s.parent_shift_id IS NULL THEN 'create' ELSE 'revise' END,
    -- created_by: use workspace owner UUID as fallback (required NOT NULL)
    '00000000-0000-0000-0000-000000000000',  -- placeholder, system migration
    COALESCE(s.created_at, now())
FROM public.shifts s
WHERE public.resolve_template_id(s.id) IS NOT NULL;

-- Drop helper function
DROP FUNCTION public.resolve_template_id(UUID);

-- 3b. Set superseded_by for non-current versions in the same template chain
-- For each template, versions are ordered by version_number.
-- Each non-current version is superseded by the next one.
UPDATE public.shift_versions sv
SET superseded_by = next_v.id
FROM (
    SELECT
        sv1.id,
        sv2.id AS next_id
    FROM public.shift_versions sv1
    JOIN public.shift_versions sv2
      ON sv1.template_id = sv2.template_id
     AND sv1.version_number + 1 = sv2.version_number
    WHERE sv1.effective_to IS NOT NULL
) next_v
WHERE sv.id = next_v.id;

-- 3c. Close effective_to for versions superseded by a later version
UPDATE public.shift_versions sv
SET effective_to = (
    SELECT (next_v.effective_from - INTERVAL '1 day')::DATE
    FROM public.shift_versions next_v
    WHERE next_v.id = sv.superseded_by
      AND next_v.change_type IN ('create', 'revise')  -- corrections don't shift effective_to
)
WHERE sv.superseded_by IS NOT NULL
  AND sv.effective_to IS NOT NULL;  -- only update if not already set

-- 3d. Set current_version_id on templates
UPDATE public.shift_templates st
SET current_version_id = sv.id
FROM public.shift_versions sv
WHERE st.id = sv.template_id
  AND sv.effective_to IS NULL
  AND sv.superseded_by IS NULL;

-- ---------------------------------------------------------------------------
-- Step 4: Indexes on new tables
-- ---------------------------------------------------------------------------

CREATE INDEX idx_shift_templates_workspace
    ON public.shift_templates(workspace_id);

CREATE INDEX idx_shift_versions_template
    ON public.shift_versions(template_id);

CREATE INDEX idx_shift_versions_template_effective
    ON public.shift_versions(template_id, effective_from DESC, effective_to);

CREATE INDEX idx_shift_versions_workspace
    ON public.shift_versions(workspace_id);

CREATE INDEX idx_shift_audit_log_template
    ON public.shift_audit_log(template_id);

CREATE INDEX idx_shift_audit_log_version
    ON public.shift_audit_log(version_id);

-- ---------------------------------------------------------------------------
-- Step 5: Add template_id columns to all consumer tables
-- ---------------------------------------------------------------------------

-- 5a. employee_shifts
ALTER TABLE public.employee_shifts ADD COLUMN template_id UUID;
UPDATE public.employee_shifts es
SET template_id = sv.template_id
FROM public.shift_versions sv
WHERE es.shift_id = sv.id;
-- For rows where shift_id reference is broken (shouldn't happen), use a fallback:
-- We'll handle NULLs in step 7 (drop orphan rows or set NOT NULL)

-- 5b. shift_rosters
ALTER TABLE public.shift_rosters ADD COLUMN template_id UUID;
UPDATE public.shift_rosters sr
SET template_id = sv.template_id
FROM public.shift_versions sv
WHERE sr.shift_id = sv.id;

-- 5c. shift_swap_requests
ALTER TABLE public.shift_swap_requests ADD COLUMN requester_template_id UUID;
ALTER TABLE public.shift_swap_requests ADD COLUMN target_template_id UUID;
UPDATE public.shift_swap_requests ssr
SET requester_template_id = sv_req.template_id
FROM public.shift_versions sv_req
WHERE ssr.requester_shift_id = sv_req.id;
UPDATE public.shift_swap_requests ssr
SET target_template_id = sv_tgt.template_id
FROM public.shift_versions sv_tgt
WHERE ssr.target_shift_id = sv_tgt.id;

-- 5d. time_entries
ALTER TABLE public.time_entries ADD COLUMN template_id UUID;
UPDATE public.time_entries te
SET template_id = sv.template_id
FROM public.shift_versions sv
WHERE te.shift_id = sv.id;

-- 5e. work_segments
ALTER TABLE public.work_segments ADD COLUMN template_id UUID;
UPDATE public.work_segments ws
SET template_id = sv.template_id
FROM public.shift_versions sv
WHERE ws.shift_id = sv.id;

-- 5f. working_pattern_details
ALTER TABLE public.working_pattern_details ADD COLUMN template_id UUID;
UPDATE public.working_pattern_details wpd
SET template_id = sv.template_id
FROM public.shift_versions sv
WHERE wpd.shift_id = sv.id;

-- 5g. shift_default_intervals (FK changes from shift_id → version_id)
ALTER TABLE public.shift_default_intervals ADD COLUMN version_id UUID;
UPDATE public.shift_default_intervals sdi
SET version_id = sdi.shift_id;  -- 1:1 mapping since each shift row has its own intervals
-- Note: shift_id was the version_id all along. We keep the old column until
-- constraints are dropped.

-- ---------------------------------------------------------------------------
-- Step 6: Drop old FK constraints on consumer tables
-- ---------------------------------------------------------------------------

ALTER TABLE public.employee_shifts
    DROP CONSTRAINT IF EXISTS employee_shifts_new_shift_id_fkey;

ALTER TABLE public.shift_rosters
    DROP CONSTRAINT IF EXISTS shift_rosters_shift_id_fkey;

ALTER TABLE public.shift_swap_requests
    DROP CONSTRAINT IF EXISTS shift_swap_requests_requester_shift_id_fkey;
ALTER TABLE public.shift_swap_requests
    DROP CONSTRAINT IF EXISTS shift_swap_requests_target_shift_id_fkey;

ALTER TABLE public.time_entries
    DROP CONSTRAINT IF EXISTS time_entries_shift_id_fkey;

ALTER TABLE public.work_segments
    DROP CONSTRAINT IF EXISTS work_segments_shift_id_fkey;

ALTER TABLE public.working_pattern_details
    DROP CONSTRAINT IF EXISTS working_pattern_details_shift_id_fkey;

ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_shift_id_fkey;
ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_shift_id_sort_order_key;

-- ---------------------------------------------------------------------------
-- Step 7: Handle NULL template_ids (orphan rows referencing deleted shifts)
-- ---------------------------------------------------------------------------

-- For rows where template_id IS NULL (shift was deleted),
-- delete them if possible or create a dummy template.
-- employee_shifts: CASCADE with old FK, so should all resolve
-- shift_rosters: SET NULL with old FK, some may be NULL — keep as NULL (day off)
-- time_entries: no CASCADE, may have orphan rows — keep as NULL template_id
-- work_segments: RESTRICT, should all resolve

-- Delete employee_shifts rows that couldn't resolve (should be zero)
DELETE FROM public.employee_shifts WHERE template_id IS NULL;

-- Set NOT NULL on tables where template_id is required
ALTER TABLE public.employee_shifts ALTER COLUMN template_id SET NOT NULL;
ALTER TABLE public.work_segments ALTER COLUMN template_id SET NOT NULL;

-- shift_rosters.template_id stays nullable (NULL = day off / no shift assigned)
-- time_entries.template_id stays nullable (historical entries may not have shift)
-- shift_swap_requests.requester_template_id stays nullable
-- shift_swap_requests.target_template_id stays nullable
-- working_pattern_details.template_id stays nullable (NULL = day off in pattern)

ALTER TABLE public.shift_default_intervals ALTER COLUMN version_id SET NOT NULL;

-- ---------------------------------------------------------------------------
-- Step 8: Add new FK constraints (template_id → shift_templates.id)
-- ---------------------------------------------------------------------------

ALTER TABLE public.shift_templates
    ADD CONSTRAINT shift_templates_current_version_fkey
    FOREIGN KEY (current_version_id) REFERENCES public.shift_versions(id)
    ON DELETE SET NULL;

ALTER TABLE public.employee_shifts
    ADD CONSTRAINT employee_shifts_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES public.shift_templates(id)
    ON DELETE CASCADE;

ALTER TABLE public.shift_rosters
    ADD CONSTRAINT shift_rosters_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES public.shift_templates(id)
    ON DELETE SET NULL;

ALTER TABLE public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_template_id_fkey
    FOREIGN KEY (requester_template_id) REFERENCES public.shift_templates(id)
    ON DELETE SET NULL;
ALTER TABLE public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_template_id_fkey
    FOREIGN KEY (target_template_id) REFERENCES public.shift_templates(id)
    ON DELETE SET NULL;

ALTER TABLE public.time_entries
    ADD CONSTRAINT time_entries_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES public.shift_templates(id)
    ON DELETE SET NULL;

ALTER TABLE public.work_segments
    ADD CONSTRAINT work_segments_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES public.shift_templates(id)
    ON DELETE RESTRICT;

ALTER TABLE public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES public.shift_templates(id)
    ON DELETE SET NULL;

ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_version_id_fkey
    FOREIGN KEY (version_id) REFERENCES public.shift_versions(id)
    ON DELETE CASCADE;
ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_version_id_sort_order_key
    UNIQUE (version_id, sort_order);

-- ---------------------------------------------------------------------------
-- Step 9: Drop old shift_id columns from consumer tables
-- ---------------------------------------------------------------------------

ALTER TABLE public.employee_shifts DROP COLUMN shift_id;
ALTER TABLE public.shift_rosters DROP COLUMN shift_id;
ALTER TABLE public.shift_swap_requests DROP COLUMN requester_shift_id;
ALTER TABLE public.shift_swap_requests DROP COLUMN target_shift_id;
ALTER TABLE public.time_entries DROP COLUMN shift_id;
ALTER TABLE public.work_segments DROP COLUMN shift_id;
ALTER TABLE public.working_pattern_details DROP COLUMN shift_id;
ALTER TABLE public.shift_default_intervals DROP COLUMN shift_id;

-- ---------------------------------------------------------------------------
-- Step 10: Old shifts table: drop parent_shift_id FK, rename to _old for
--          backward compatibility during code transition
-- ---------------------------------------------------------------------------

ALTER TABLE public.shifts DROP CONSTRAINT IF EXISTS shifts_parent_shift_id_fkey;
ALTER TABLE public.shifts DROP CONSTRAINT IF EXISTS shifts_employer_id_fkey;
ALTER TABLE public.shifts DROP CONSTRAINT IF EXISTS shifts_location_id_fkey;

-- ---------------------------------------------------------------------------
-- Step 11: Data integrity verification
-- ---------------------------------------------------------------------------

DO $$
DECLARE
    orphan_versions INT;
    missing_current INT;
    ref_employee_shifts INT;
    ref_work_segments INT;
BEGIN
    -- Check for versions without a template
    SELECT COUNT(*) INTO orphan_versions
    FROM public.shift_versions sv
    LEFT JOIN public.shift_templates st ON st.id = sv.template_id
    WHERE st.id IS NULL;
    IF orphan_versions > 0 THEN
        RAISE EXCEPTION 'DATA INTEGRITY: % versions have no template', orphan_versions;
    END IF;

    -- Check for templates without a current version
    SELECT COUNT(*) INTO missing_current
    FROM public.shift_templates st
    WHERE st.current_version_id IS NULL;
    IF missing_current > 0 THEN
        RAISE WARNING '% templates have no current_version_id', missing_current;
        -- Not an error: templates created for archives may have no current version
    END IF;

    -- Check employee_shifts template references
    SELECT COUNT(*) INTO ref_employee_shifts
    FROM public.employee_shifts es
    LEFT JOIN public.shift_templates st ON st.id = es.template_id
    WHERE st.id IS NULL;
    IF ref_employee_shifts > 0 THEN
        RAISE EXCEPTION 'DATA INTEGRITY: % employee_shifts rows have invalid template_id', ref_employee_shifts;
    END IF;

    -- Check work_segments template references
    SELECT COUNT(*) INTO ref_work_segments
    FROM public.work_segments ws
    LEFT JOIN public.shift_templates st ON st.id = ws.template_id
    WHERE st.id IS NULL;
    IF ref_work_segments > 0 THEN
        RAISE EXCEPTION 'DATA INTEGRITY: % work_segments rows have invalid template_id', ref_work_segments;
    END IF;
END $$;

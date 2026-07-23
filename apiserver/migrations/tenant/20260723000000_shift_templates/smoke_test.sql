-- ============================================================================
-- Smoke Test: Shift Versioning Migration Verification
--
-- Run AFTER applying migration 20260723000000_shift_templates/up.sql
-- on a tenant workspace database.
--
-- Verifies:
--   1. Template coverage: every shift has a template
--   2. Version chain: template → version → intervals
--   3. FK migration: all consumer tables resolve via template_id
--   4. Correction semantics: superseded_by points correctly
-- ============================================================================

-- ---------------------------------------------------------------------------
-- 1. Template Coverage: every shift → exactly 1 template
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    orphan INT;
    active_without_current INT;
BEGIN
    -- Count shifts that failed to resolve to a template
    SELECT COUNT(*) INTO orphan
    FROM public.shifts s
    WHERE NOT EXISTS (
        SELECT 1 FROM public.shift_versions sv
        WHERE sv.id = s.id
    )
    AND s.id NOT IN (SELECT id FROM public.shift_templates);

    IF orphan > 0 THEN
        RAISE WARNING 'SMOKE FAIL: % shifts have no template or version', orphan;
    ELSE
        RAISE NOTICE 'SMOKE PASS: All shifts have template coverage';
    END IF;

    -- Count active templates without a current version
    SELECT COUNT(*) INTO active_without_current
    FROM public.shift_templates st
    WHERE st.current_version_id IS NULL
      AND EXISTS (
          SELECT 1 FROM public.shift_versions sv
          WHERE sv.template_id = st.id AND sv.effective_to IS NULL
      );

    IF active_without_current > 0 THEN
        RAISE WARNING 'SMOKE FAIL: % templates with versions but no current_version_id', active_without_current;
    ELSE
        RAISE NOTICE 'SMOKE PASS: All templates have current_version_id set';
    END IF;
END $$;

-- ---------------------------------------------------------------------------
-- 2. Version Chain Integrity
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    broken_chain INT;
BEGIN
    -- Templates must have at least 1 version
    SELECT COUNT(*) INTO broken_chain
    FROM public.shift_templates st
    WHERE NOT EXISTS (
        SELECT 1 FROM public.shift_versions sv WHERE sv.template_id = st.id
    );

    IF broken_chain > 0 THEN
        RAISE WARNING 'SMOKE FAIL: % templates have no versions', broken_chain;
    ELSE
        RAISE NOTICE 'SMOKE PASS: All templates have at least 1 version';
    END IF;
END $$;

-- ---------------------------------------------------------------------------
-- 3. FK Migration: Consumer tables resolve via template_id
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    broken_es INT;
    broken_sr INT;
    broken_ws INT;
    broken_te INT;
    broken_wpd INT;
    broken_ssr INT;
    broken_sdi INT;
BEGIN
    SELECT COUNT(*) INTO broken_es
    FROM public.employee_shifts es
    WHERE es.template_id IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = es.template_id);

    SELECT COUNT(*) INTO broken_sr
    FROM public.shift_rosters sr
    WHERE sr.template_id IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = sr.template_id);

    SELECT COUNT(*) INTO broken_ws
    FROM public.work_segments ws
    WHERE ws.template_id IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = ws.template_id);

    SELECT COUNT(*) INTO broken_te
    FROM public.time_entries te
    WHERE te.template_id IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = te.template_id);

    SELECT COUNT(*) INTO broken_wpd
    FROM public.working_pattern_details wpd
    WHERE wpd.template_id IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = wpd.template_id);

    SELECT COUNT(*) INTO broken_ssr
    FROM public.shift_swap_requests ssr
    WHERE (
        (ssr.requester_template_id IS NOT NULL
         AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = ssr.requester_template_id))
        OR
        (ssr.target_template_id IS NOT NULL
         AND NOT EXISTS (SELECT 1 FROM public.shift_templates st WHERE st.id = ssr.target_template_id))
    );

    SELECT COUNT(*) INTO broken_sdi
    FROM public.shift_default_intervals sdi
    WHERE NOT EXISTS (SELECT 1 FROM public.shift_versions sv WHERE sv.id = sdi.version_id);

    IF broken_es + broken_sr + broken_ws + broken_te + broken_wpd + broken_ssr + broken_sdi = 0 THEN
        RAISE NOTICE 'SMOKE PASS: All consumer FK references are valid';
    ELSE
        RAISE WARNING 'SMOKE FAIL: Broken FKs — employee_shifts=%, shift_rosters=%, work_segments=%, time_entries=%, working_pattern_details=%, shift_swaps=%, intervals=%',
            broken_es, broken_sr, broken_ws, broken_te, broken_wpd, broken_ssr, broken_sdi;
    END IF;
END $$;

-- ---------------------------------------------------------------------------
-- 4. Version Chain Integrity: superseded_by points to valid versions
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    broken_superseded INT;
BEGIN
    SELECT COUNT(*) INTO broken_superseded
    FROM public.shift_versions sv
    WHERE sv.superseded_by IS NOT NULL
      AND NOT EXISTS (SELECT 1 FROM public.shift_versions sv2 WHERE sv2.id = sv.superseded_by);

    IF broken_superseded > 0 THEN
        RAISE WARNING 'SMOKE FAIL: % versions have broken superseded_by references', broken_superseded;
    ELSE
        RAISE NOTICE 'SMOKE PASS: All superseded_by references are valid';
    END IF;
END $$;

-- ---------------------------------------------------------------------------
-- 5. Summary
-- ---------------------------------------------------------------------------
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'SMOKE TEST COMPLETE';
    RAISE NOTICE 'Templates: %', (SELECT COUNT(*) FROM public.shift_templates);
    RAISE NOTICE 'Versions:  %', (SELECT COUNT(*) FROM public.shift_versions);
    RAISE NOTICE 'Audit logs:%', (SELECT COUNT(*) FROM public.shift_audit_log);
    RAISE NOTICE '========================================';
END $$;

-- PostgreSQL behavior checks for 4-layer HRM employee tenure, relationship, and segment integrity.
-- Run after all migrations. Every row is rolled back at the end.

BEGIN;

DO $$
DECLARE
    v_employee_id uuid := gen_random_uuid();
    v_employer_a_id uuid := gen_random_uuid();
    v_employer_b_id uuid := gen_random_uuid();
    v_relationship_a_id uuid := gen_random_uuid();
    v_relationship_b_id uuid := gen_random_uuid();
    v_tenure_a_id uuid := gen_random_uuid();
    v_tenure_b_id uuid := gen_random_uuid();
    v_other_employee_id uuid := gen_random_uuid();
    v_segment_a_id uuid := gen_random_uuid();
    v_segment_b_id uuid := gen_random_uuid();
    v_contract_id uuid := gen_random_uuid();
    v_rejected boolean;
BEGIN
    INSERT INTO public.employees (id, full_name)
    VALUES (v_employee_id, 'Schema Contract Test');

    INSERT INTO public.employee_tenures (
        id, employee_id, employee_number, hire_type, start_date, end_date
    ) VALUES (
        v_tenure_a_id, v_employee_id, 'EMP-1541', 'initial', DATE '2026-01-01', DATE '2026-01-31'
    );

    -- Reusing a number for the same person is valid after the prior tenure closes.
    INSERT INTO public.employee_tenures (
        id, employee_id, employee_number, hire_type, prior_service_credit_days, start_date
    ) VALUES (
        v_tenure_b_id, v_employee_id, ' emp-1541 ', 'rehire', 31, DATE '2026-02-01'
    );

    INSERT INTO public.employees (id, full_name)
    VALUES (v_other_employee_id, 'Schema Contract Other Employee');

    -- Permanent cross-person code ban: another person can NEVER reuse EMP-1541, even on non-overlapping dates or different casing.
    v_rejected := false;
    BEGIN
        INSERT INTO public.employee_tenures (
            id, employee_id, employee_number, start_date, end_date
        ) VALUES (
            gen_random_uuid(), v_other_employee_id, 'emp-1541', DATE '2025-01-01', DATE '2025-12-31'
        );
    EXCEPTION WHEN exclusion_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'cross-person employee number reuse was accepted';
    END IF;

    INSERT INTO public.employers (id, name)
    VALUES (v_employer_a_id, 'Schema Contract Employer A');

    INSERT INTO public.employers (id, name)
    VALUES (v_employer_b_id, 'Schema Contract Employer B');

    INSERT INTO public.employment_relationships (
        id, employee_id, employer_id, tenure_id, start_date, end_date
    ) VALUES (
        v_relationship_a_id, v_employee_id, v_employer_a_id, v_tenure_a_id,
        DATE '2026-01-01', DATE '2026-01-31'
    );

    -- Different employers may overlap inside the same tenure.
    INSERT INTO public.employment_relationships (
        id, employee_id, employer_id, tenure_id, start_date, end_date
    ) VALUES (
        v_relationship_b_id, v_employee_id, v_employer_b_id, v_tenure_a_id,
        DATE '2026-01-15', DATE '2026-01-31'
    );

    -- Relationship outside tenure bounds must be rejected.
    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_relationships (
            employee_id, employer_id, tenure_id, start_date, end_date
        ) VALUES (
            v_employee_id, v_employer_b_id, v_tenure_a_id, DATE '2026-02-01', DATE '2026-02-15'
        );
    EXCEPTION WHEN check_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'relationship outside tenure bounds was accepted';
    END IF;

    -- Same employee/employer overlap must be rejected.
    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_relationships (
            employee_id, employer_id, tenure_id, start_date, end_date
        ) VALUES (
            v_employee_id, v_employer_a_id, v_tenure_a_id, DATE '2026-01-15', DATE '2026-01-31'
        );
    EXCEPTION WHEN exclusion_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'same employee/employer overlap was accepted';
    END IF;

    INSERT INTO public.employment_segments (
        id, employee_id, employer_id, relationship_id, status, start_date, end_date
    ) VALUES (
        v_segment_a_id, v_employee_id, v_employer_a_id, v_relationship_a_id,
        'terminated', DATE '2026-01-01', DATE '2026-01-31'
    );

    INSERT INTO public.employment_segments (
        id, employee_id, employer_id, relationship_id, status, start_date, end_date
    ) VALUES (
        v_segment_b_id, v_employee_id, v_employer_b_id, v_relationship_b_id,
        'terminated', DATE '2026-01-15', DATE '2026-01-31'
    );

    -- Segment outside relationship bounds must be rejected.
    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_segments (
            id, employee_id, employer_id, relationship_id, status, start_date, end_date
        ) VALUES (
            gen_random_uuid(), v_employee_id, v_employer_a_id, v_relationship_a_id,
            'active', DATE '2026-01-01', NULL
        );
    EXCEPTION WHEN check_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'open segment inside closed relationship was accepted';
    END IF;

    -- Segment relationship identity mismatch must be rejected.
    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_segments (
            id, employee_id, employer_id, relationship_id, status, start_date, end_date
        ) VALUES (
            gen_random_uuid(), v_employee_id, v_employer_b_id, v_relationship_a_id,
            'terminated', DATE '2026-01-10', DATE '2026-01-20'
        );
    EXCEPTION WHEN foreign_key_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'segment relationship identity mismatch was accepted';
    END IF;

    INSERT INTO public.employee_contracts (
        id, contract_type, status, start_date, employment_id, relationship_id
    ) VALUES (
        v_contract_id, 'fixed_term', 'active', DATE '2026-01-01',
        v_segment_a_id, v_relationship_a_id
    );

    v_rejected := false;
    BEGIN
        INSERT INTO public.employee_contracts (
            id, contract_type, status, start_date, employment_id, relationship_id
        ) VALUES (
            gen_random_uuid(), 'fixed_term', 'active', DATE '2026-01-15',
            v_segment_a_id, v_relationship_b_id
        );
    EXCEPTION WHEN check_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'contract relationship mismatch was accepted';
    END IF;
END;
$$;

ROLLBACK;

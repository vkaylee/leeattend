-- PostgreSQL behavior checks for employment relationship integrity.
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
    v_tenure_conflict_id uuid := gen_random_uuid();
    v_segment_a_id uuid := gen_random_uuid();
    v_segment_b_id uuid := gen_random_uuid();
    v_contract_id uuid := gen_random_uuid();
    v_rejected boolean;
BEGIN
    INSERT INTO public.employees (
        id, employee_id_number, full_name, hire_date, original_hire_date
    ) VALUES (
        v_employee_id, 'schema-contract-test-' || replace(v_employee_id::text, '-', ''),
        'Schema Contract Test', DATE '2026-01-01', DATE '2026-01-01'
    );

    INSERT INTO public.employee_tenures (
        id, employee_id, employee_number, start_date, end_date
    ) VALUES (
        v_tenure_a_id, v_employee_id, '1541', DATE '2026-01-01', DATE '2026-01-31'
    );

    -- Reusing a number for the same person is valid after the prior tenure closes.
    INSERT INTO public.employee_tenures (
        id, employee_id, employee_number, start_date
    ) VALUES (
        v_tenure_b_id, v_employee_id, '1541', DATE '2026-02-01'
    );

    INSERT INTO public.employees (
        id, employee_id_number, full_name, hire_date, original_hire_date
    ) VALUES (
        v_other_employee_id, 'schema-contract-other-' || replace(v_other_employee_id::text, '-', ''),
        'Schema Contract Other Employee', DATE '2026-01-01', DATE '2026-01-01'
    );

    v_rejected := false;
    BEGIN
        INSERT INTO public.employee_tenures (
            id, employee_id, employee_number, start_date, end_date
        ) VALUES (
            v_tenure_conflict_id, v_other_employee_id, '1541', DATE '2026-01-15', DATE '2026-02-15'
        );
    EXCEPTION WHEN exclusion_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'employee number overlap across people was accepted';
    END IF;

    INSERT INTO public.employers (id, name)
    VALUES (v_employer_a_id, 'Schema Contract Employer A');

    INSERT INTO public.employers (id, name)
    VALUES (v_employer_b_id, 'Schema Contract Employer B');

    INSERT INTO public.employment_relationships (
        id, employee_id, employer_id, start_date, end_date
    ) VALUES (
        v_relationship_a_id, v_employee_id, v_employer_a_id,
        DATE '2026-01-01', DATE '2026-01-31'
    );

    -- Different employers may overlap for one employee.
    INSERT INTO public.employment_relationships (
        id, employee_id, employer_id, start_date, end_date
    ) VALUES (
        v_relationship_b_id, v_employee_id, v_employer_b_id,
        DATE '2026-01-15', DATE '2026-02-15'
    );

    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_relationships (
            employee_id, employer_id, start_date, end_date
        ) VALUES (
            v_employee_id, v_employer_a_id, DATE '2026-01-31', DATE '2026-02-01'
        );
    EXCEPTION WHEN exclusion_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'same employee/employer overlap was accepted';
    END IF;

    v_rejected := false;
    BEGIN
        INSERT INTO public.employment_relationships (
            employee_id, employer_id, start_date, end_date
        ) VALUES (
            v_employee_id, v_employer_a_id, DATE '2026-03-01', DATE '2026-02-28'
        );
    EXCEPTION WHEN check_violation THEN
        v_rejected := true;
    END;
    IF NOT v_rejected THEN
        RAISE EXCEPTION 'invalid relationship dates were accepted';
    END IF;

    INSERT INTO public.employments (
        id, employee_id, employer_id, relationship_id, status, start_date
    ) VALUES (
        v_segment_a_id, v_employee_id, v_employer_a_id, v_relationship_a_id,
        'active', DATE '2026-01-01'
    );

    INSERT INTO public.employments (
        id, employee_id, employer_id, relationship_id, status, start_date
    ) VALUES (
        v_segment_b_id, v_employee_id, v_employer_b_id, v_relationship_b_id,
        'active', DATE '2026-01-15'
    );

    v_rejected := false;
    BEGIN
        INSERT INTO public.employments (
            id, employee_id, employer_id, relationship_id, status, start_date
        ) VALUES (
            gen_random_uuid(), v_employee_id, v_employer_b_id, v_relationship_a_id,
            'active', DATE '2026-03-01'
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

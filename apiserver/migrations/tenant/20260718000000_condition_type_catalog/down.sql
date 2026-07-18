-- Reverse Phase 1: Condition Type Catalog

-- 1. Remove condition_type_id from working_conditions
ALTER TABLE public.working_conditions
    DROP CONSTRAINT IF EXISTS working_conditions_condition_type_id_fkey;

DROP INDEX IF EXISTS idx_working_conditions_condition_type_id;

ALTER TABLE public.working_conditions
    DROP COLUMN IF EXISTS condition_type_id;

-- 2. Drop condition_type_definitions table
DROP TABLE IF EXISTS public.condition_type_definitions CASCADE;

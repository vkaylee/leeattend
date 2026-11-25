BEGIN;

ALTER TABLE public.employee_profiles
    ADD COLUMN national_id_lookup_hash character(64),
    ADD COLUMN phone_lookup_hash character(64);

ALTER TABLE public.employee_profiles
    ADD CONSTRAINT employee_profiles_identity_lookup_hash_check
    CHECK (
        (national_id_lookup_hash IS NULL OR national_id_lookup_hash::text ~ '^[0-9a-f]{64}$')
        AND (phone_lookup_hash IS NULL OR phone_lookup_hash::text ~ '^[0-9a-f]{64}$')
    );

-- Keep lookup indexes non-unique until populated duplicate report / backfill collision queue is clean.
CREATE INDEX idx_employee_profiles_national_id_lookup_hash
    ON public.employee_profiles (national_id_lookup_hash)
    WHERE national_id_lookup_hash IS NOT NULL;

CREATE INDEX idx_employee_profiles_phone_lookup_hash
    ON public.employee_profiles (phone_lookup_hash)
    WHERE phone_lookup_hash IS NOT NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261125000000_employee_identity_matching');

COMMIT;

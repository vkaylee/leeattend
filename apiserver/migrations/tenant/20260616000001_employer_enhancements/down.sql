ALTER TABLE public.employers
    DROP COLUMN IF EXISTS business_type,
    DROP COLUMN IF EXISTS registration_number,
    DROP COLUMN IF EXISTS phone,
    DROP COLUMN IF EXISTS email,
    DROP COLUMN IF EXISTS website,
    DROP COLUMN IF EXISTS incorporation_date,
    DROP COLUMN IF EXISTS country_code,
    DROP COLUMN IF EXISTS currency_code;

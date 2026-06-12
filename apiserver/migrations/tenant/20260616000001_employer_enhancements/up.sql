ALTER TABLE public.employers
    ADD COLUMN business_type VARCHAR(50),            -- 'llc', 'corporation', 'sole_proprietorship', 'partnership'
    ADD COLUMN registration_number VARCHAR(100),     -- business registration number
    ADD COLUMN phone VARCHAR(20),
    ADD COLUMN email VARCHAR(255),
    ADD COLUMN website VARCHAR(255),
    ADD COLUMN incorporation_date DATE,
    ADD COLUMN country_code VARCHAR(3),              -- ISO 3166-1 alpha-3
    ADD COLUMN currency_code VARCHAR(3) DEFAULT 'VND';

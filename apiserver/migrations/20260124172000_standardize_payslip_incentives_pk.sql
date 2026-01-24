-- Up Migration: Standardize payslip_incentives PK from SERIAL to UUID

-- 1. Add new UUID column
ALTER TABLE payslip_incentives ADD COLUMN IF NOT EXISTS new_id UUID DEFAULT gen_random_uuid();

-- 2. Populate new_id for existing rows
UPDATE payslip_incentives SET new_id = gen_random_uuid() WHERE new_id IS NULL;

-- 3. Drop old primary key constraint
ALTER TABLE payslip_incentives DROP CONSTRAINT IF EXISTS payslip_incentives_pkey;

-- 4. Drop old SERIAL id column
ALTER TABLE payslip_incentives DROP COLUMN IF EXISTS id;

-- 5. Rename new_id to id
ALTER TABLE payslip_incentives RENAME COLUMN new_id TO id;

-- 6. Add primary key constraint on new UUID id
ALTER TABLE payslip_incentives ADD PRIMARY KEY (id);

-- 7. Set NOT NULL and default
ALTER TABLE payslip_incentives ALTER COLUMN id SET NOT NULL;
ALTER TABLE payslip_incentives ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- Down Migration (commented):
-- Note: Reversing this migration would require recreating SERIAL which loses original IDs
-- ALTER TABLE payslip_incentives DROP CONSTRAINT payslip_incentives_pkey;
-- ALTER TABLE payslip_incentives DROP COLUMN id;
-- ALTER TABLE payslip_incentives ADD COLUMN id SERIAL PRIMARY KEY;

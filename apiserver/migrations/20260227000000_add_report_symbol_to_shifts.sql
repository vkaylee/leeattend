-- Add optional report symbol to shifts (max 10 chars, shown in Excel cell prefix)
ALTER TABLE shifts ADD COLUMN IF NOT EXISTS report_symbol VARCHAR(10) NULL;

CREATE TYPE early_arrival_strategy_type AS ENUM ('allow', 'block', 'clamp');
ALTER TABLE workspaces ADD COLUMN early_arrival_strategy early_arrival_strategy_type NOT NULL DEFAULT 'block';

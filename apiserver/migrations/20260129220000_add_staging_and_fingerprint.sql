-- Phase 1: Dual-Layer Protection & Staging Area
-- Creates staging table for unknown users and adds fingerprint to error logs

-- ============================================
-- Action A: Create attendance_staging_logs table
-- ============================================
-- Purpose: Store valid logs from devices where user is unknown
-- These can be reviewed/mapped later without causing agent retries

CREATE TABLE IF NOT EXISTS attendance_staging_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    device_id TEXT,
    raw_user_id TEXT NOT NULL,  -- Unknown user ID from device
    punch_time TIMESTAMPTZ NOT NULL,
    raw_data JSONB,
    fingerprint TEXT NOT NULL,  -- Hash for deduplication
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, processed, dismissed
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Unique index for deduplication (prevents duplicate spam)
CREATE UNIQUE INDEX IF NOT EXISTS idx_staging_fingerprint
ON attendance_staging_logs(workspace_id, fingerprint);

-- Index for querying pending logs efficiently
CREATE INDEX IF NOT EXISTS idx_staging_pending
ON attendance_staging_logs(workspace_id, status) WHERE status = 'pending';

-- Index for workspace queries
CREATE INDEX IF NOT EXISTS idx_staging_workspace
ON attendance_staging_logs(workspace_id);

-- ============================================
-- Action B: Add fingerprint to attendance_log_errors
-- ============================================
-- Purpose: Prevent duplicate error spam from validation failures

ALTER TABLE attendance_log_errors
ADD COLUMN IF NOT EXISTS fingerprint TEXT;

-- Unique index for deduplication (partial - only where fingerprint exists)
-- This allows existing rows without fingerprint to remain valid
CREATE UNIQUE INDEX IF NOT EXISTS idx_errors_fingerprint
ON attendance_log_errors(workspace_id, fingerprint)
WHERE fingerprint IS NOT NULL;

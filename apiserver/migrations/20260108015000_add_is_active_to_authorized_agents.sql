-- Migration: Add is_active to authorized_agents table
ALTER TABLE authorized_agents ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;
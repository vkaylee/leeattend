-- Add migration script here
CREATE SCHEMA IF NOT EXISTS _supavisor;

CREATE TABLE IF NOT EXISTS _supavisor.tenants (
    id BIGSERIAL PRIMARY KEY,
    external_id VARCHAR(255) NOT NULL UNIQUE,
    db_host VARCHAR(255) NOT NULL,
    db_port INTEGER NOT NULL,
    db_database VARCHAR(255) NOT NULL,
    db_user VARCHAR(255) NOT NULL,
    db_password VARCHAR(255) NOT NULL,
    default_pool_size INTEGER NOT NULL DEFAULT 15,
    pool_mode VARCHAR(50) NOT NULL DEFAULT 'transaction',
    allow_roles TEXT[] NOT NULL DEFAULT '{}',
    inserted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

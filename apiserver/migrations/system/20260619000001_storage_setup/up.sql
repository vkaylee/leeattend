-- Up migration: create system_files and alter users table
CREATE TABLE public.system_files (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name       VARCHAR(255) NOT NULL,
    storage_key     VARCHAR(500) NOT NULL UNIQUE,
    file_size       BIGINT NOT NULL,
    mime_type       VARCHAR(100) NOT NULL,
    content_hash    VARCHAR(64),
    
    category        VARCHAR(50) NOT NULL,  -- 'user_avatar', 'system_asset'
    owner_type      VARCHAR(50) NOT NULL,  -- 'user', 'system'
    owner_id        UUID NOT NULL,
    
    uploaded_by     UUID,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    scan_status     VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    version         INT NOT NULL DEFAULT 1,
    parent_file_id  UUID REFERENCES public.system_files(id),
    is_latest       BOOLEAN NOT NULL DEFAULT true,
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sf_owner ON public.system_files(owner_type, owner_id) WHERE is_active = true;

ALTER TABLE public.users ADD COLUMN avatar_file_id UUID REFERENCES public.system_files(id);

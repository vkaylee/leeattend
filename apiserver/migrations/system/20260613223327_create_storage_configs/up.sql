CREATE TABLE public.storage_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL UNIQUE REFERENCES public.workspaces(id) ON DELETE CASCADE,
    
    endpoint_url VARCHAR(255) NOT NULL,
    region VARCHAR(50) NOT NULL,
    bucket_name VARCHAR(100) NOT NULL,
    path_prefix VARCHAR(255),
    
    access_key_enc VARCHAR(500) NOT NULL,
    secret_key_enc VARCHAR(500) NOT NULL,
    encryption_key_version VARCHAR(50) NOT NULL,
    
    use_path_style BOOLEAN NOT NULL DEFAULT true,
    presigned_ttl_secs INTEGER NOT NULL DEFAULT 900,
    max_file_size BIGINT NOT NULL DEFAULT 10485760,
    
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_updated_at_storage_configs 
    BEFORE UPDATE ON public.storage_configs 
    FOR EACH ROW 
    EXECUTE FUNCTION public.fn_set_updated_at();

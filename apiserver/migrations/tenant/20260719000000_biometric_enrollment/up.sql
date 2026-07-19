-- ============================================================================
-- Biometric Enrollment
-- Enables HR to manage which attendance devices an employee is enrolled on.
--
-- Two decoupled concepts:
--   1. employee_biometric_templates — server-side source-of-truth store for
--      fingerprint templates (encrypted at rest). Push to any device reads
--      from here.
--   2. device_enrollments — membership + sync state: "this employee is (to be)
--      present on this device". This is what HR manipulates directly.
--
-- NOTE: workspace_id is a logical scoping column (database-per-tenant); there
-- is no FK to a workspace table in the tenant DB.
-- ============================================================================

-- 1. Biometric template store (source of truth)
CREATE TABLE public.employee_biometric_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    finger_id smallint NOT NULL,
    template_encrypted text NOT NULL,
    encryption_key_version character varying(50) NOT NULL,
    source_device_id uuid,
    quality smallint,
    valid boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT employee_biometric_templates_pkey PRIMARY KEY (id),
    CONSTRAINT employee_biometric_templates_employee_finger_key UNIQUE (employee_id, finger_id),
    CONSTRAINT employee_biometric_templates_finger_id_range CHECK (finger_id >= 0 AND finger_id <= 9)
);

ALTER TABLE ONLY public.employee_biometric_templates
    ADD CONSTRAINT employee_biometric_templates_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_biometric_templates
    ADD CONSTRAINT employee_biometric_templates_source_device_id_fkey
        FOREIGN KEY (source_device_id) REFERENCES public.devices(id) ON DELETE SET NULL;

CREATE INDEX idx_employee_biometric_templates_workspace_employee
    ON public.employee_biometric_templates USING btree (workspace_id, employee_id);

COMMENT ON TABLE public.employee_biometric_templates IS 'Server-side source-of-truth store of fingerprint templates, encrypted at rest (AES-256-GCM). Push to devices reads from here.';
COMMENT ON COLUMN public.employee_biometric_templates.finger_id IS 'Finger index 0-9 (ZK fid).';
COMMENT ON COLUMN public.employee_biometric_templates.template_encrypted IS 'AES-256-GCM payload: {key_ver}:{nonce_b64}:{ct_b64}. Raw fingerprint bytes; PII T1, never returned raw via API.';
COMMENT ON COLUMN public.employee_biometric_templates.source_device_id IS 'Device where this template was originally enrolled (provenance/trace).';

-- 2. Device enrollment (membership + sync state)
CREATE TABLE public.device_enrollments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    device_id uuid NOT NULL,
    device_uid integer,
    sync_status character varying(30) DEFAULT 'pending_push'::character varying NOT NULL,
    last_synced_at timestamp with time zone,
    last_error text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT device_enrollments_pkey PRIMARY KEY (id),
    CONSTRAINT device_enrollments_employee_device_key UNIQUE (employee_id, device_id),
    CONSTRAINT device_enrollments_sync_status_check CHECK (
        (sync_status)::text = ANY (ARRAY[
            'pending_push'::text,
            'synced'::text,
            'pending_removal'::text,
            'failed'::text,
            'removed'::text
        ])
    )
);

ALTER TABLE ONLY public.device_enrollments
    ADD CONSTRAINT device_enrollments_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.device_enrollments
    ADD CONSTRAINT device_enrollments_device_id_fkey
        FOREIGN KEY (device_id) REFERENCES public.devices(id) ON DELETE CASCADE;

CREATE INDEX idx_device_enrollments_workspace_device_status
    ON public.device_enrollments USING btree (workspace_id, device_id, sync_status);

CREATE INDEX idx_device_enrollments_employee
    ON public.device_enrollments USING btree (employee_id);

COMMENT ON TABLE public.device_enrollments IS 'Membership + sync state: which employees are (to be) present on which devices. This is what HR manipulates directly.';
COMMENT ON COLUMN public.device_enrollments.device_uid IS 'The u16 uid the device assigned to this user; null until first successful push. Per-device — same employee has different uid on different devices.';
COMMENT ON COLUMN public.device_enrollments.sync_status IS 'State machine: pending_push -> synced; synced -> pending_removal -> removed; any -> failed.';

-- 3. Link devices to a work location (filter/quick-select only, nullable)
ALTER TABLE public.devices
    ADD COLUMN location_id uuid;

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_location_id_fkey
        FOREIGN KEY (location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

CREATE INDEX idx_devices_location_id
    ON public.devices USING btree (location_id);

COMMENT ON COLUMN public.devices.location_id IS 'Optional work location, used only as a filter/quick-select hint for bulk enrollment. Not the source of truth for enrollment.';

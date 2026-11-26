BEGIN;

-- Fresh-install-only rollback for foundation; run phases in reverse order.

-- This is destructive and must not run against production data.

DROP TABLE IF EXISTS public.working_conditions CASCADE;

DROP TABLE IF EXISTS public.work_locations CASCADE;

DROP TABLE IF EXISTS public.users CASCADE;

DROP TABLE IF EXISTS public.user_roles CASCADE;

DROP TABLE IF EXISTS public.user_extra_roles CASCADE;

DROP TABLE IF EXISTS public.user_extra_permissions CASCADE;

DROP TABLE IF EXISTS public.system_settings CASCADE;

DROP TABLE IF EXISTS public.storage_configs CASCADE;

DROP TABLE IF EXISTS public.signature_requests CASCADE;

DROP TABLE IF EXISTS public.sessions CASCADE;

DROP TABLE IF EXISTS public.schema_migrations CASCADE;

DROP TABLE IF EXISTS public.roles CASCADE;

DROP TABLE IF EXISTS public.role_permissions CASCADE;

DROP TABLE IF EXISTS public.outbox_jobs CASCADE;

DROP TABLE IF EXISTS public.organization_settings CASCADE;

DROP TABLE IF EXISTS public.notifications CASCADE;

DROP TABLE IF EXISTS public.monitor_alerts CASCADE;

DROP TABLE IF EXISTS public.monitor_alert_events CASCADE;

DROP TABLE IF EXISTS public.lifecycle_checklists CASCADE;

DROP TABLE IF EXISTS public.lifecycle_checklist_templates CASCADE;

DROP TABLE IF EXISTS public.lifecycle_checklist_template_items CASCADE;

DROP TABLE IF EXISTS public.lifecycle_checklist_items CASCADE;

DROP TABLE IF EXISTS public.infra_job_records CASCADE;

DROP TABLE IF EXISTS public.infra_job_events CASCADE;

DROP TABLE IF EXISTS public.files CASCADE;

DROP TABLE IF EXISTS public.file_acknowledgements CASCADE;

DROP TABLE IF EXISTS public.feature_flags CASCADE;

DROP TABLE IF EXISTS public.document_folders CASCADE;

DROP TABLE IF EXISTS public.dashboard_preferences CASCADE;

DROP TABLE IF EXISTS public.custom_field_values CASCADE;

DROP TABLE IF EXISTS public.custom_field_definitions CASCADE;

DROP TABLE IF EXISTS public.config_entries CASCADE;

DROP TABLE IF EXISTS public.condition_type_definitions CASCADE;

DROP TABLE IF EXISTS public.compliance_rules CASCADE;

DROP TABLE IF EXISTS public.behalf_assignments CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m12 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m11 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m10 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m09 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m08 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m07 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m06 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m05 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m04 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m03 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m02 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_y2026m01 CASCADE;

DROP TABLE IF EXISTS public.audit_logs_default CASCADE;

DROP TABLE IF EXISTS public.audit_logs CASCADE;

DROP TABLE IF EXISTS public.approval_workflows CASCADE;

DROP TABLE IF EXISTS public.approval_workflow_tasks CASCADE;

DROP TABLE IF EXISTS public.approval_workflow_steps CASCADE;

DROP TABLE IF EXISTS public.approval_workflow_instances CASCADE;

DROP TABLE IF EXISTS public.approval_action_log CASCADE;

DROP SCHEMA public CASCADE;

CREATE SCHEMA public;

COMMIT;

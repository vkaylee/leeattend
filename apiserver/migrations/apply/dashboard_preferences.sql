BEGIN;
INSERT INTO public.schema_migrations(version)
VALUES ('20261107000000_add_window_layout_to_dashboard_preferences');
\ir ../20261107000000_add_window_layout_to_dashboard_preferences/up.sql

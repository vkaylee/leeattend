DELETE FROM public.dashboard_preferences older
USING public.dashboard_preferences newer
WHERE older.user_id = newer.user_id
  AND older.workspace_id = newer.workspace_id
  AND (
      older.updated_at < newer.updated_at
      OR (older.updated_at = newer.updated_at AND older.id < newer.id)
  );

CREATE UNIQUE INDEX dashboard_preferences_user_workspace_unique
    ON public.dashboard_preferences (user_id, workspace_id);

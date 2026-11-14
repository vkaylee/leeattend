import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Flag, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { SettingsSwitch } from "@/components/ui/settings/settings-switch";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchFeatureFlags, updateFeatureFlag, type FeatureFlag } from "@/lib/api";
import { FEATURE_FLAG_CATALOG, setRuntimeFeatureFlags, useFeatureFlagVersion } from "@/lib/feature-flags";

const queryKey = ["feature-flags"] as const;

function FeatureFlagSkeleton() {
  return (
    <div className="space-y-3" data-testid="feature-flags-skeleton">
      {[1, 2, 3].map((item) => <Skeleton className="h-24 w-full rounded-2xl" key={item} />)}
    </div>
  );
}

function FeatureFlagRow({ flag, canManage }: { flag: FeatureFlag; canManage: boolean }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (enabled: boolean) => updateFeatureFlag(flag.id, enabled),
    onMutate: async (enabled) => {
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<FeatureFlag[]>(queryKey);
      queryClient.setQueryData<FeatureFlag[]>(queryKey, (current) => current?.map((item) => item.id === flag.id ? { ...item, enabled } : item));
      return { previous };
    },
    onError: (_error, _enabled, context) => queryClient.setQueryData(queryKey, context?.previous),
    onSuccess: (updated) => {
      const current = queryClient.getQueryData<FeatureFlag[]>(queryKey) ?? [];
      setRuntimeFeatureFlags(current.map((item) => item.id === updated.id ? { key: item.flag_key, enabled: updated.enabled } : { key: item.flag_key, enabled: item.enabled }));
    },
    onSettled: () => { void queryClient.invalidateQueries({ queryKey }); },
  });

  const handleToggle = (enabled: boolean) => {
    if (!canManage || mutation.isPending) return;
    mutation.mutate(enabled);
  };
  const catalogEntry = Object.prototype.hasOwnProperty.call(FEATURE_FLAG_CATALOG, flag.flag_key)
    ? (FEATURE_FLAG_CATALOG as Record<string, { description: string }>)[flag.flag_key]
    : undefined;

  return (
    <>
      <div className="flex flex-col gap-4 rounded-2xl border border-slate-200/80 bg-white p-5 shadow-2xs dark:border-slate-800/80 dark:bg-slate-900/90 sm:flex-row sm:items-center sm:justify-between" data-testid={`feature-flag-${flag.flag_key}`}>
        <div className="min-w-0">
          <Typography className="font-semibold text-slate-800 dark:text-slate-100" variant="h3">{flag.flag_key}</Typography>
          <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">{flag.description ?? catalogEntry?.description ?? t("feature_flags.no_description", { defaultValue: "No description" })}</Typography>
          <Typography className="mt-2 text-xs text-slate-400 dark:text-slate-500" variant="p">{t("feature_flags.updated", { defaultValue: "Updated" })}: {new Date(flag.updated_at).toLocaleString()}</Typography>
        </div>
        <div className="flex items-center gap-3 self-end sm:self-auto">
          {catalogEntry ? null : <span className="text-xs text-amber-600 dark:text-amber-400">{t("feature_flags.unregistered", { defaultValue: "Unregistered consumer" })}</span>}
          <span className="text-sm font-medium text-slate-600 dark:text-slate-300">{flag.enabled ? t("common.enabled", { defaultValue: "Enabled" }) : t("feature_flags.disabled", { defaultValue: "Disabled" })}</span>
          <SettingsSwitch ariaLabel={`${flag.flag_key} ${t("feature_flags.toggle", { defaultValue: "toggle" })}`} checked={flag.enabled} disabled={!canManage || mutation.isPending} onCheckedChange={handleToggle} testId={`feature-flag-toggle-${flag.flag_key}`} />
        </div>
      </div>
    </>
  );
}

export default function FeatureFlagsPage() {
  useFeatureFlagVersion();
  const { t } = useTranslation();
  const { hasPermission } = useMyPermissions();
  const query = useQuery({ queryKey, queryFn: fetchFeatureFlags, meta: { suppressErrorToast: true } });
  const canManage = hasPermission("feature_flags:manage");

  if (query.isLoading) return <FeatureFlagSkeleton />;
  if (query.isError) {
    return <EmptyState action={<Button icon={<RefreshCw size={15} />} onClick={() => void query.refetch()} variant="primary">{t("common.retry", { defaultValue: "Retry" })}</Button>} description={t("feature_flags.load_error", { defaultValue: "Unable to load feature flags." })} icon={<Flag className="h-10 w-10" />} title={t("feature_flags.title", { defaultValue: "Feature flags" })} />;
  }
  if (!query.data?.length) return <EmptyState description={t("feature_flags.empty", { defaultValue: "No feature flags configured." })} icon={<Flag className="h-10 w-10" />} title={t("feature_flags.title", { defaultValue: "Feature flags" })} />;

  return (
    <div className="space-y-6">
      <SettingsHeader description={t("feature_flags.description", { defaultValue: "Control runtime features for this organization." })} icon={<Flag size={20} />} title={<Typography className="text-xl font-bold text-slate-800 dark:text-slate-100" variant="h1">{t("feature_flags.title", { defaultValue: "Feature flags" })}</Typography>} />
      <div aria-label={t("feature_flags.title", { defaultValue: "Feature flags" })} className="space-y-3">
        {query.data.map((flag) => <FeatureFlagRow canManage={canManage} flag={flag} key={flag.id} />)}
      </div>
    </div>
  );
}

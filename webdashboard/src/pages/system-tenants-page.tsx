import { Building2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import PageHeader from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useSystemTenants } from "@/hooks/use-system-tenants";

export default function SystemTenantsPage() {
  const { t } = useTranslation();
  const { tenants, isLoading, isError } = useSystemTenants();

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <PageHeader
        icon={<Building2 className="h-5 w-5" />}
        subtitle={t("system_admin.tenants.subtitle", "Manage platform workspaces and tenant database mode.")}
        title={t("system_admin.tenants.title", "Tenants")}
      />
      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2">
          {["tenant-skeleton-1", "tenant-skeleton-2"].map((key) => <Skeleton className="h-32 rounded-2xl" key={key} />)}
        </div>
      ) : isError ? (
        <div aria-live="assertive" className="rounded-2xl border border-rose-200 bg-rose-50 p-6 text-sm text-rose-700" role="alert">
          {t("common.error", "Unable to load tenants.")}
        </div>
      ) : tenants.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-300 p-10 text-center">
          <Typography className="text-slate-500" variant="p">{t("common.no_data", "No data found")}</Typography>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {tenants.map((tenant) => (
            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-950" key={tenant.id}>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="font-semibold text-slate-900 dark:text-slate-100">{tenant.name}</h2>
                  <p className="mt-1 text-sm text-slate-500">{tenant.timezone}</p>
                </div>
                <span className="rounded-full bg-slate-100 px-2 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300">
                  {tenant.is_byod ? "BYOD" : "Managed"}
                </span>
              </div>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}

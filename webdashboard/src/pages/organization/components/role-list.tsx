import { ShieldCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";

import type { OrganizationRole } from "../../../lib/api/types";

import { RoleIcon, RoleListActions } from "./role-list-subcomponents";

interface RoleListProps {
  activeMenu: string | null;
  isLoading: boolean;
  onDelete: (id: string) => void;
  onEdit: (role: OrganizationRole) => void;
  roles: OrganizationRole[];
  setActiveMenu: (id: string | null) => void;
}

function RoleListSkeleton() {
  return (
    <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900" data-testid="roles-list-skeleton">
      {[0, 1, 2].map((row) => (
        <div className="flex items-center gap-4 border-b border-slate-100 p-4 last:border-b-0 dark:border-slate-800" key={row}>
          <Skeleton className="h-10 w-10 shrink-0 rounded-xl" />
          <div className="min-w-0 flex-1 space-y-2">
            <Skeleton className="h-4 w-36" />
            <Skeleton className="h-3 w-64 max-w-full" />
          </div>
          <Skeleton className="h-8 w-8 rounded-lg" />
        </div>
      ))}
    </div>
  );
}

export function RoleList({
  activeMenu,
  isLoading,
  onDelete,
  onEdit,
  roles,
  setActiveMenu,
}: RoleListProps) {
  const { t } = useTranslation();

  if (isLoading) return <RoleListSkeleton />;

  if (roles.length === 0) {
    return (
      <EmptyState
        description={t("rolesPage.emptyDescription", { defaultValue: "Create a role to define reusable access levels for your organization." })}
        icon={<ShieldCheck className="h-10 w-10" />}
        title={t("rolesPage.emptyTitle", { defaultValue: "No roles yet" })}
      />
    );
  }

  return (
    <div
      aria-label={t("rolesPage.listLabel", { defaultValue: "Organization roles" })}
      className="overflow-x-auto rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900"
      role="list"
    >
      <div
        aria-hidden="true"
        className="hidden grid-cols-[minmax(14rem,1.2fr)_minmax(16rem,2fr)_auto] gap-4 border-b border-slate-200 bg-slate-50/80 px-5 py-3 text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:border-slate-800 dark:bg-slate-950/40 dark:text-slate-400 md:grid"
      >
        <span>{t("rolesPage.roleColumn", { defaultValue: "Role" })}</span>
        <span>{t("rolesPage.descriptionColumn", { defaultValue: "Description" })}</span>
        <span className="sr-only">{t("common.actions", "Actions")}</span>
      </div>

      <div className="divide-y divide-slate-100 dark:divide-slate-800">
        {roles.map((role) => (
          <div
            className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4 px-4 py-4 transition-colors hover:bg-slate-50/70 dark:hover:bg-slate-800/40 md:grid-cols-[minmax(14rem,1.2fr)_minmax(16rem,2fr)_auto] md:px-5"
            key={role.id}
            role="listitem"
          >
            <div className="flex min-w-0 items-center gap-3">
              <RoleIcon role={role} />
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="truncate text-sm font-bold text-slate-800 dark:text-slate-100">{role.name}</h3>
                  <span className="rounded-md bg-slate-100 px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider text-slate-500 dark:bg-slate-800 dark:text-slate-400">
                    {role.is_system
                      ? t("common.system", { defaultValue: "System" })
                      : t("rolesPage.customRole", { defaultValue: "Custom" })}
                  </span>
                </div>
                <p className="mt-1 truncate text-xs text-slate-500 dark:text-slate-400 md:hidden">
                  {role.description ?? t("rolesPage.noDescription", { defaultValue: "No description provided." })}
                </p>
              </div>
            </div>

            <p className="hidden truncate text-sm text-slate-500 dark:text-slate-400 md:block">
              {role.description ?? t("rolesPage.noDescription", { defaultValue: "No description provided." })}
            </p>

            <div className="justify-self-end">
              <RoleListActions
                activeMenu={activeMenu}
                onDelete={onDelete}
                onEdit={onEdit}
                role={role}
                setActiveMenu={setActiveMenu}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

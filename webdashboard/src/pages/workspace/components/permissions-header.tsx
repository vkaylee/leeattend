import { Save, SlidersHorizontal } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import PageHeader from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

import type { WorkspaceRole } from "../../../lib/api/types";

interface PermissionsHeaderProps {
  hasChanges: boolean;
  hiddenRoleIds: string[];
  isSaving: boolean;
  onSave: () => void;
  roles: WorkspaceRole[];
  setHiddenRoleIds: React.Dispatch<React.SetStateAction<string[]>>;
}

export function PermissionsHeader({
  hasChanges,
  isSaving,
  roles,
  hiddenRoleIds,
  setHiddenRoleIds,
  onSave,
}: PermissionsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <PageHeader
        subtitle={t("permissions.subtitle", {
          defaultValue: "Fine-tune access control for each role across all modules",
        })}
        title={t("permissions.title", { defaultValue: "Permission Matrix" })}
      />
      <div className="flex items-center gap-3">
        {hasChanges ? (
          <span className="text-xs font-bold text-amber-500 animate-pulse">
            {t("permissions.unsavedChanges", { defaultValue: "Unsaved changes" })}
          </span>
        ) : null}

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button className="flex items-center gap-2 px-4 py-2.5 rounded-2xl font-bold text-sm bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors shadow-sm">
              <SlidersHorizontal className="text-slate-500" size={16} />
              {t("common.columns", { defaultValue: "Columns" })}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
            <DropdownMenuLabel>
              {t("permissions.visibleRoles", { defaultValue: "Visible Roles" })}
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            {roles.map((role) => (
              <DropdownMenuCheckboxItem
                checked={!hiddenRoleIds.includes(role.id)}
                key={role.id}
                onCheckedChange={(checked) => {
                  setHiddenRoleIds((previous) =>
                    checked ? previous.filter((id) => id !== role.id) : [...previous, role.id]
                  );
                }}
              >
                {t(`roles.${role.name}`, { defaultValue: role.name })}
                {role.is_system ? (
                  <span className="ml-2 text-[9px] text-slate-400 font-normal uppercase tracking-widest bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded">
                    {t("common.system", { defaultValue: "System" })}
                  </span>
                ) : null}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        <PermissionGate permission="members:roles:manage">
          <Button
            className={cn(
              "flex items-center gap-2 px-5 py-2.5 rounded-2xl font-bold text-sm transition-all",
              hasChanges
                ? "bg-primary-600 hover:bg-primary-700 text-white shadow-lg shadow-primary-200 dark:shadow-primary-900/20 hover:-translate-y-0.5"
                : "bg-slate-100 dark:bg-slate-800 text-slate-400 cursor-not-allowed"
            )}
            disabled={!hasChanges || isSaving}
            onClick={onSave}
            type="button"
          >
            {isSaving ? (
              <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              <Save size={18} />
            )}
            {t("permissions.saveMatrix", { defaultValue: "Save Matrix" })}
          </Button>
        </PermissionGate>
      </div>
    </div>
  );
}

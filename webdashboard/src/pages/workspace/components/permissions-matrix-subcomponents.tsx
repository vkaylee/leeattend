import { Check, ChevronLeft, ChevronRight, Info, KeyRound } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import type { WorkspaceRole } from "../../../lib/api/types";

interface ScrollVisualCuesProps {
  canScrollLeft: boolean;
  canScrollRight: boolean;
}

export const ScrollVisualCues = ({ canScrollLeft, canScrollRight }: ScrollVisualCuesProps) => (
  <>
    <div
      className={cn(
        "absolute left-[280px] top-0 bottom-0 w-12 bg-gradient-to-r from-slate-200/50 dark:from-slate-950/50 to-transparent z-10 pointer-events-none transition-opacity duration-300",
        canScrollLeft ? "opacity-100" : "opacity-0"
      )}
    />
    <div
      className={cn(
        "absolute right-0 top-0 bottom-0 w-12 bg-gradient-to-l from-slate-200/50 dark:from-slate-950/50 to-transparent z-10 pointer-events-none transition-opacity duration-300",
        canScrollRight ? "opacity-100" : "opacity-0"
      )}
    />
  </>
);

interface ScrollButtonsProps {
  canScrollLeft: boolean;
  canScrollRight: boolean;
  scrollBy: (amount: number) => void;
}

export const ScrollButtons = ({ canScrollLeft, canScrollRight, scrollBy }: ScrollButtonsProps) => (
  <div className="sticky top-[30vh] z-30 h-0 w-full pointer-events-none">
    <Button
      className={cn(
        "absolute left-[265px] top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto",
        canScrollLeft ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none"
      )}
      onClick={() => { scrollBy(-300); }}
      type="button"
    >
      <ChevronLeft size={20} />
    </Button>

    <Button
      className={cn(
        "absolute right-4 top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto",
        canScrollRight ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none"
      )}
      onClick={() => { scrollBy(300); }}
      type="button"
    >
      <ChevronRight size={20} />
    </Button>
  </div>
);

interface PermissionTableHeaderProps {
  visibleRoles: WorkspaceRole[];
}

export const PermissionTableHeader = ({ visibleRoles }: PermissionTableHeaderProps) => {
  const { t } = useTranslation();
  
  return (
    <thead>
      <tr>
        <th className="sticky left-0 z-20 bg-slate-50 dark:bg-slate-900 border-b border-r border-slate-100 dark:border-slate-800 p-6 min-w-[280px]">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-50 dark:bg-blue-900/30 rounded-xl">
              <KeyRound className="w-5 h-5 text-blue-500" />
            </div>
            <div>
              <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="h3">
                {t("permissions.modules", { defaultValue: "Modules" })}
              </Typography>
              <Typography className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5" variant="p">
                {t("permissions.permissionsLabel", { defaultValue: "Permissions" })}
              </Typography>
            </div>
          </div>
        </th>
        {visibleRoles.map((role) => (
          <th
            className="bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 p-6 text-center min-w-[140px]"
            key={role.id}
          >
            <div className="flex flex-col items-center justify-center gap-2">
              <Typography className="font-bold text-sm text-slate-800 dark:text-slate-200 truncate w-full px-2"
                title={t(`roles.${role.name}`, { defaultValue: role.name })}
                variant="h4"
              >
                {t(`roles.${role.name}`, { defaultValue: role.name })}
              </Typography>
              {role.is_system ? (
                <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500">
                  {t("common.system", { defaultValue: "System" })}
                </span>
              ) : (
                <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                  {t("common.custom", { defaultValue: "Custom" })}
                </span>
              )}
            </div>
          </th>
        ))}
      </tr>
    </thead>
  );
};

interface PermissionTableRowProps {
  hasManagePermission: boolean;
  perm: { id: string; name: string; desc: string };
  permissions: Record<string, string[]>;
  togglePermission: (role: WorkspaceRole, permId: string) => void;
  visibleRoles: WorkspaceRole[];
}

export const PermissionTableRow = ({
  perm,
  visibleRoles,
  permissions,
  hasManagePermission,
  togglePermission,
}: PermissionTableRowProps) => {
  const { t } = useTranslation();
  
  return (
    <tr className="group hover:bg-slate-50 dark:hover:bg-slate-800/30 transition-colors">
      <td className="sticky left-0 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 border-b border-r border-slate-100 dark:border-slate-800 p-4 pl-6">
        <div className="flex items-start gap-2">
          <div>
            <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="p">
              {t(`permissions.items.${perm.id}.name`, { defaultValue: perm.name })}
            </Typography>
            <Typography className="text-xs text-slate-500 mt-0.5" variant="p">
              {t(`permissions.items.${perm.id}.desc`, { defaultValue: perm.desc })}
            </Typography>
          </div>
        </div>
      </td>

      {visibleRoles.map((role) => {
        const rolePermissions = (permissions[role.id] as string[] | undefined) ?? [];
        const isChecked = rolePermissions.includes(perm.id);
        const isSystem = role.is_system;

        return (
          <td
            className="border-b border-slate-100 dark:border-slate-800 p-4 text-center"
            key={`${role.id}-${perm.id}`}
          >
            <label
              className={cn(
                "relative inline-flex items-center justify-center w-6 h-6 rounded-lg border-2 transition-all",
                isSystem || !hasManagePermission
                  ? "cursor-not-allowed opacity-60"
                  : "cursor-pointer hover:border-primary-400",
                isChecked
                  ? "bg-primary-500 border-primary-500 text-white"
                  : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
              )}
            >
              <input
                checked={isChecked}
                className="sr-only"
                disabled={isSystem || !hasManagePermission}
                onChange={() => { togglePermission(role, perm.id); }}
                type="checkbox"
              />
              <Check className="w-4 h-4" strokeWidth={3} />
            </label>
          </td>
        );
      })}
    </tr>
  );
};

export const PermissionMatrixFooter = () => {
  const { t } = useTranslation();
  
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 p-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-center gap-2">
      <Info className="w-4 h-4 text-slate-400" />
      <Typography className="text-xs font-medium text-slate-500" variant="p">
        {t("permissions.systemRoleInfo", {
          defaultValue: "System roles (like Admin) have fixed permissions that cannot be modified.",
        })}
      </Typography>
    </div>
  );
};

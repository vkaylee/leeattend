import { Check, ChevronDown, ChevronLeft, ChevronRight, Info, KeyRound } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { permissionGroupIcons } from "@/lib/permission-groups";
import { cn } from "@/lib/utils";

import type { PermissionCatalogItem, OrganizationRole } from "../../../lib/api/types";

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
    <Button className={cn(
        "absolute left-[265px] top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto",
        canScrollLeft ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none"
      )}
      onClick={() => { scrollBy(-300); }}
      type="button"
      variant="ghost"
    >
      <ChevronLeft size={20} />
    </Button>

    <Button className={cn(
        "absolute right-4 top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto",
        canScrollRight ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none"
      )}
      onClick={() => { scrollBy(300); }}
      type="button"
      variant="ghost"
    >
      <ChevronRight size={20} />
    </Button>
  </div>
);

interface PermissionTableHeaderProps {
  visibleRoles: OrganizationRole[];
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

export interface PermissionCatalogGroup {
  id: string;
  permissions: PermissionCatalogItem[];
}

interface PermissionToggleProps {
  ariaLabel: string;
  checked: boolean;
  disabled: boolean;
  onChange: () => void;
  testId?: string;
}

export const PermissionToggle = ({ ariaLabel, checked, disabled, onChange, testId }: PermissionToggleProps) => (
  <label
    className={cn(
      "relative inline-flex items-center justify-center w-7 h-7 rounded-lg border-2 transition-all",
      disabled
        ? "cursor-not-allowed opacity-60"
        : "cursor-pointer hover:border-primary-400",
      checked
        ? "bg-primary-500 border-primary-500 text-white"
        : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent",
    )}
  >
    <input
      aria-label={ariaLabel}
      checked={checked}
      className="sr-only"
      data-testid={testId}
      disabled={disabled}
      onChange={onChange}
      type="checkbox"
    />
    <Check className="w-4 h-4" strokeWidth={3} />
  </label>
);

interface PermissionTableRowProps {
  hasManagePermission: boolean;
  perm: PermissionCatalogItem;
  permissions: Record<string, string[]>;
  togglePermission: (role: OrganizationRole, permId: string) => void;
  visibleRoles: OrganizationRole[];
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
              {t(perm.label_key, { defaultValue: perm.id })}
            </Typography>
            <Typography className="text-xs text-slate-500 mt-0.5" variant="p">
              {t(perm.label_key.replace(/\.name$/, ".desc"), { defaultValue: perm.id })}
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
            <PermissionToggle
              ariaLabel={t(perm.label_key, { defaultValue: perm.id })}
              checked={isChecked}
              disabled={isSystem || !hasManagePermission}
              onChange={() => { togglePermission(role, perm.id); }}
              testId={`permission-toggle-${role.id}-${perm.id}`}
            />
          </td>
        );
      })}
    </tr>
  );
};

interface MobilePermissionRoleSectionProps {
  defaultExpanded: boolean;
  hasManagePermission: boolean;
  permissionGroups: PermissionCatalogGroup[];
  permissions: Record<string, string[]>;
  role: OrganizationRole;
  togglePermission: (role: OrganizationRole, permId: string) => void;
}

export const MobilePermissionRoleSection = ({
  defaultExpanded,
  hasManagePermission,
  permissions,
  permissionGroups,
  role,
  togglePermission,
}: MobilePermissionRoleSectionProps) => {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const roleLabel = t(`roles.${role.name}`, { defaultValue: role.name });
  const permissionCount = useMemo(
    () => permissionGroups.reduce((count, group) => count + group.permissions.length, 0),
    [permissionGroups],
  );
  const grantedCount = permissionGroups.reduce(
    (count, group) => count + group.permissions.filter((perm) => permissions[role.id].includes(perm.id)).length,
    0,
  );
  const contentId = `permissions-mobile-role-content-${role.id}`;

  return (
    <section className="border-b border-slate-100 last:border-b-0 dark:border-slate-800" data-testid={`permissions-mobile-role-${role.id}`}>
      <Button
        aria-controls={contentId}
        aria-expanded={isExpanded}
        className="flex min-h-[56px] w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-slate-50 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500 dark:hover:bg-slate-800/50 justify-start h-auto"
        onClick={() => { setIsExpanded((expanded) => !expanded); }}
        type="button"
        variant="ghost"
      >
        <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-primary-50 text-sm font-bold text-primary-600 dark:bg-primary-950 dark:text-primary-300">
          {roleLabel.charAt(0).toUpperCase()}
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-bold text-slate-800 dark:text-slate-200">{roleLabel}</span>
          <span className="mt-0.5 block text-[10px] font-bold uppercase tracking-widest text-slate-400">
            {role.is_system
              ? t("common.system", { defaultValue: "System" })
              : t("common.custom", { defaultValue: "Custom" })}
            <span aria-hidden="true"> · </span>
            {t("permissions.mobilePermissionCount", {
              count: grantedCount,
              total: permissionCount,
              defaultValue: "{{count}} of {{total}} enabled",
            })}
          </span>
        </span>
        <ChevronDown
          aria-hidden="true"
          className={cn("h-5 w-5 shrink-0 text-slate-400 transition-transform duration-200", isExpanded && "rotate-180")}
        />
      </Button>

      <div className={cn("px-4 pb-4", !isExpanded && "hidden")} id={contentId}>
        {permissionGroups.map((group) => {
          const GroupIcon = permissionGroupIcons[group.id] ?? KeyRound;
          return (
            <div className="pt-3 first:pt-0" key={group.id}>
              <div className="mb-2 flex items-center gap-2 text-[10px] font-bold uppercase tracking-widest text-slate-400">
                <GroupIcon aria-hidden="true" className="h-3.5 w-3.5" />
                <span>{t(`permissions.groups.${group.id}`, { defaultValue: group.id })}</span>
              </div>
              <div className="space-y-2">
                {group.permissions.map((perm) => {
                  const isChecked = permissions[role.id].includes(perm.id);
                  const isDisabled = role.is_system || !hasManagePermission;
                  const permissionLabel = t(perm.label_key, { defaultValue: perm.id });
                  return (
                    <div
                      className="flex min-h-[52px] items-center gap-3 rounded-xl border border-slate-100 bg-slate-50/60 px-3 py-2 dark:border-slate-800 dark:bg-slate-800/30"
                      data-testid={`permissions-mobile-permission-${role.id}-${perm.id}`}
                      key={perm.id}
                    >
                      <div className="min-w-0 flex-1">
                        <Typography variant="p" className="truncate text-sm font-bold text-slate-700 dark:text-slate-200">{permissionLabel}</Typography>
                        <Typography variant="p" className="mt-0.5 line-clamp-2 text-xs text-slate-500 dark:text-slate-400">
                          {t(perm.label_key.replace(/\.name$/, ".desc"), { defaultValue: perm.id })}
                        </Typography>
                      </div>
                      <PermissionToggle
                        ariaLabel={`${roleLabel}: ${permissionLabel}`}
                        checked={isChecked}
                        disabled={isDisabled}
                        onChange={() => { togglePermission(role, perm.id); }}
                        testId={`permissions-mobile-toggle-${role.id}-${perm.id}`}
                      />
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </section>
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

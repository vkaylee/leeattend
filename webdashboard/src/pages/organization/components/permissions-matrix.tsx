import { KeyRound } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { permissionGroupIcons } from "@/lib/permission-groups";

import type { PermissionCatalogItem, OrganizationRole } from "../../../lib/api/types";
import type { PermissionCatalogGroup } from "./permissions-matrix-subcomponents";

import {
  MobilePermissionRoleSection,
  PermissionMatrixFooter,
  PermissionTableHeader,
  PermissionTableRow,
  ScrollButtons,
  ScrollVisualCues,
} from "./permissions-matrix-subcomponents";

interface PermissionsMatrixProps {
  canScrollLeft: boolean;
  canScrollRight: boolean;
  catalog: PermissionCatalogItem[];
  handleScroll: () => void;
  hasManagePermission: boolean;
  isCatalogError: boolean;
  isCatalogLoading: boolean;
  isMatrixLoading: boolean;
  isRolesLoading: boolean;
  permissions: Record<string, string[]>;
  scrollBy: (amount: number) => void;
  scrollContainerRef: React.RefObject<HTMLDivElement | null>;
  togglePermission: (role: OrganizationRole, permId: string) => void;
  visibleRoles: OrganizationRole[];
}

export function PermissionsMatrix(props: PermissionsMatrixProps) {
  const { t } = useTranslation();
  const {
    catalog,
    isCatalogError,
    isCatalogLoading,
    isRolesLoading,
    isMatrixLoading,
    canScrollLeft,
    canScrollRight,
    visibleRoles,
    permissions,
    hasManagePermission,
    scrollContainerRef,
    handleScroll,
    scrollBy,
    togglePermission,
  } = props;

  const permissionGroups = useMemo<PermissionCatalogGroup[]>(() => {
    const groups: PermissionCatalogGroup[] = [];
    for (const permission of catalog) {
      const group = groups.find((candidate) => candidate.id === permission.group);
      if (group) {
        group.permissions.push(permission);
      } else {
        groups.push({ id: permission.group, permissions: [permission] });
      }
    }
    return groups;
  }, [catalog]);

  if (isCatalogError) {
    return (
      <div className="p-12 text-center text-sm font-medium text-rose-600 dark:text-rose-400" role="alert">
        {t("permissions.catalog_load_error", {
          defaultValue: "Permission catalog could not be loaded. Please try again.",
        })}
      </div>
    );
  }

  if (isRolesLoading || isMatrixLoading || isCatalogLoading) {
    return (
      <div className="flex justify-center p-20">
        <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
      </div>
    );
  }

  return (
    <div className="relative">
      <div className="group/table hidden lg:block" data-testid="permissions-desktop">
        <ScrollVisualCues canScrollLeft={canScrollLeft} canScrollRight={canScrollRight} />
        <ScrollButtons canScrollLeft={canScrollLeft} canScrollRight={canScrollRight} scrollBy={scrollBy} />

        <div className="overflow-x-auto custom-scrollbar relative z-0" onScroll={handleScroll} ref={scrollContainerRef}>
          <table className="w-full text-left border-collapse min-w-[800px]">
            <PermissionTableHeader visibleRoles={visibleRoles} />
            <tbody>
              {permissionGroups.map((group) => {
                const GroupIcon = permissionGroupIcons[group.id] ?? KeyRound;

                return (
                  <React.Fragment key={group.id}>
                    <tr>
                      <td
                        className="bg-slate-50/50 dark:bg-slate-800/20 border-b border-slate-100 dark:border-slate-800 py-3 px-6"
                        colSpan={visibleRoles.length + 1}
                      >
                        <div className="flex items-center gap-2">
                          <GroupIcon className="w-4 h-4 text-slate-400" />
                          <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                            {t(`permissions.groups.${group.id}`, { defaultValue: group.id })}
                          </span>
                        </div>
                      </td>
                    </tr>

                    {group.permissions.map((perm) => (
                      <PermissionTableRow
                        hasManagePermission={hasManagePermission}
                        key={perm.id}
                        perm={perm}
                        permissions={permissions}
                        togglePermission={togglePermission}
                        visibleRoles={visibleRoles}
                      />
                    ))}
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      <div className="lg:hidden" data-testid="permissions-mobile">
        {visibleRoles.length > 0 ? visibleRoles.map((role, index) => (
          <MobilePermissionRoleSection
            defaultExpanded={index === 0}
            hasManagePermission={hasManagePermission}
            key={role.id}
            permissionGroups={permissionGroups}
            permissions={permissions}
            role={role}
            togglePermission={togglePermission}
          />
        )) : (
          <Typography className="p-6 text-center text-sm text-slate-500 dark:text-slate-400" variant="p">
            {t("permissions.no_roles", { defaultValue: "No roles available." })}
          </Typography>
        )}
      </div>

      <PermissionMatrixFooter />
    </div>
  );
}

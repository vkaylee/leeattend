import React from "react";
import { useTranslation } from "react-i18next";

import { permissionGroups } from "@/lib/permission-groups";

import type { WorkspaceRole } from "../../../lib/api/types";

import {
  PermissionMatrixFooter,
  PermissionTableHeader,
  PermissionTableRow,
  ScrollButtons,
  ScrollVisualCues,
} from "./permissions-matrix-subcomponents";

interface PermissionsMatrixProps {
  canScrollLeft: boolean;
  canScrollRight: boolean;
  handleScroll: () => void;
  hasManagePermission: boolean;
  isMatrixLoading: boolean;
  isRolesLoading: boolean;
  permissions: Record<string, string[]>;
  scrollBy: (amount: number) => void;
  scrollContainerRef: React.RefObject<HTMLDivElement | null>;
  togglePermission: (role: WorkspaceRole, permId: string) => void;
  visibleRoles: WorkspaceRole[];
}

export function PermissionsMatrix(props: PermissionsMatrixProps) {
  const { t } = useTranslation();
  const {
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

  if (isRolesLoading || isMatrixLoading) {
    return (
      <div className="flex justify-center p-20">
        <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
      </div>
    );
  }

  return (
    <div className="relative group/table">
      <ScrollVisualCues canScrollLeft={canScrollLeft} canScrollRight={canScrollRight} />
      <ScrollButtons canScrollLeft={canScrollLeft} canScrollRight={canScrollRight} scrollBy={scrollBy} />

      <div className="overflow-x-auto custom-scrollbar relative z-0" onScroll={handleScroll} ref={scrollContainerRef}>
        <table className="w-full text-left border-collapse min-w-[800px]">
          <PermissionTableHeader visibleRoles={visibleRoles} />
          <tbody>
            {permissionGroups.map((group) => (
              <React.Fragment key={group.id}>
                <tr>
                  <td
                    className="bg-slate-50/50 dark:bg-slate-800/20 border-b border-slate-100 dark:border-slate-800 py-3 px-6"
                    colSpan={visibleRoles.length + 1}
                  >
                    <div className="flex items-center gap-2">
                      <group.icon className="w-4 h-4 text-slate-400" />
                      <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                        {t(`permissions.groups.${group.id}`, { defaultValue: group.name })}
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
            ))}
          </tbody>
        </table>
      </div>

      <PermissionMatrixFooter />
    </div>
  );
}

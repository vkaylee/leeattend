import React from "react";

import { PermissionsHeader } from "./components/permissions-header";
import { PermissionsMatrix } from "./components/permissions-matrix";
import { useWorkspacePermissionsPage } from "./hooks/use-workspace-permissions-page";

export default function WorkspacePermissionsPage() {
  const {
    wsId,
    t,
    roles,
    isRolesLoading,
    isMatrixLoading,
    hasManagePermission,
    permissions,
    hasChanges,
    isSaving,
    hiddenRoleIds,
    setHiddenRoleIds,
    scrollContainerRef,
    canScrollLeft,
    canScrollRight,
    visibleRoles,
    handleScroll,
    scrollBy,
    togglePermission,
    handleSave,
  } = useWorkspacePermissionsPage();

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-10">
      <PermissionsHeader
        hasChanges={hasChanges}
        hiddenRoleIds={hiddenRoleIds}
        isSaving={isSaving}
        onSave={handleSave}
        roles={roles}
        setHiddenRoleIds={setHiddenRoleIds}
      />
      
      <div className="bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden">
        <PermissionsMatrix
          canScrollLeft={canScrollLeft}
          canScrollRight={canScrollRight}
          handleScroll={handleScroll}
          hasManagePermission={hasManagePermission}
          isMatrixLoading={isMatrixLoading}
          isRolesLoading={isRolesLoading}
          permissions={permissions}
          scrollBy={scrollBy}
          scrollContainerRef={scrollContainerRef}
          togglePermission={togglePermission}
          visibleRoles={visibleRoles}
        />
      </div>
    </div>
  );
}

import React from "react";

import { PermissionsHeader } from "./components/permissions-header";
import { PermissionsMatrix } from "./components/permissions-matrix";
import { useOrganizationPermissionsPage } from "./hooks/use-organization-permissions-page";

export default function OrganizationPermissionsPage() {
  const {
    t,
    roles,
    catalog,
    isCatalogError,
    isCatalogLoading,
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
  } = useOrganizationPermissionsPage();

  

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
          catalog={catalog}
          isCatalogError={isCatalogError}
          isCatalogLoading={isCatalogLoading}
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

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { usePermissionCatalog, useOrganizationRoles, useOrganizationPermissionsMatrix } from "@/hooks/use-organization";
import { updateRolePermissions } from "@/lib/api";
import { logError } from "@/lib/logger";

import type { OrganizationRole } from "../../../lib/api/types";

export function useOrganizationPermissionsPage() {
  const { t } = useTranslation();
  
  const { roles, isLoading: isRolesLoading } = useOrganizationRoles();
  const {
    catalog,
    isError: isCatalogError,
    isLoading: isCatalogLoading,
  } = usePermissionCatalog();
  const { matrix, isLoading: isMatrixLoading, refetch } = useOrganizationPermissionsMatrix();
  const { hasPermission } = useMyPermissions();
  const hasManagePermission = hasPermission("members:roles:manage");

  const [permissions, setPermissions] = useState<Record<string, string[]>>(matrix);
  const [hasChanges, setHasChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [hiddenRoleIds, setHiddenRoleIds] = useState<string[]>([]);

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(true);

  const visibleRoles = roles.filter((r) => !hiddenRoleIds.includes(r.id));

  const handleScroll = useCallback(() => {
    if (scrollContainerRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollContainerRef.current;
      setCanScrollLeft(scrollLeft > 0);
      setCanScrollRight(scrollLeft + clientWidth < scrollWidth - 1);
    }
  }, []);

  useEffect(() => {
    handleScroll();
    window.addEventListener("resize", handleScroll);
    return () => {
      window.removeEventListener("resize", handleScroll);
    };
  }, [handleScroll, visibleRoles.length, roles.length]);

  const scrollBy = (amount: number) => {
    if (scrollContainerRef.current) {
      scrollContainerRef.current.scrollBy({ left: amount, behavior: "smooth" });
    }
  };

  const [prevMatrix, setPrevMatrix] = useState(matrix);
  if (matrix !== prevMatrix) {
    setPrevMatrix(matrix);
    if (Object.keys(matrix).length > 0) {
      setPermissions(matrix);
      setHasChanges(false);
    }
  }

  const togglePermission = (role: OrganizationRole, permId: string) => {
    if (role.is_system || !hasManagePermission) return;

    setPermissions((previous) => {
      const rolePerms = (previous[role.id] as string[] | undefined) ?? [];
      const newPerms = rolePerms.includes(permId) 
        ? rolePerms.filter((p) => p !== permId) 
        : [...rolePerms, permId];

      setHasChanges(true);
      return { ...previous, [role.id]: newPerms };
    });
  };

  const handleSave = async () => {
    
    setIsSaving(true);
    try {
      const modifiedRoles = roles.filter((role) => {
        const initial = (matrix[role.id] as string[] | undefined) ?? [];
        const current = (permissions[role.id] as string[] | undefined) ?? [];
        return initial.length !== current.length || !initial.every((p) => current.includes(p));
      });

      await Promise.all(
        modifiedRoles.map((role) => updateRolePermissions(role.id, (permissions[role.id] as string[] | undefined) ?? []))
      );

      setHasChanges(false);
      void refetch();
    } catch (e: unknown) {
      logError(e, { component: "useOrganizationPermissionsPage" });
    } finally {
      setIsSaving(false);
    }
  };

  return {
    t,
    roles,
    matrix,
    catalog,
    isCatalogError,
    isCatalogLoading,
    isRolesLoading,
    isMatrixLoading,
    hasManagePermission,
    permissions,
    setPermissions,
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
  };
}

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspaceRoles, useWorkspacePermissionsMatrix } from "@/hooks/use-workspace";
import { updateRolePermissions, getCurrentWorkspaceId } from "@/lib/api";
import { logError } from "@/lib/logger";

import type { WorkspaceRole } from "../../../lib/api/types";

export function useWorkspacePermissionsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { roles, isLoading: isRolesLoading } = useWorkspaceRoles(wsId ?? undefined);
  const { matrix, isLoading: isMatrixLoading, refetch } = useWorkspacePermissionsMatrix(wsId ?? undefined);
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

  const togglePermission = (role: WorkspaceRole, permId: string) => {
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
    if (!wsId) return;
    setIsSaving(true);
    try {
      const modifiedRoles = roles.filter((role) => {
        const initial = (matrix[role.id] as string[] | undefined) ?? [];
        const current = (permissions[role.id] as string[] | undefined) ?? [];
        return initial.length !== current.length || !initial.every((p) => current.includes(p));
      });

      await Promise.all(
        modifiedRoles.map((role) => updateRolePermissions(wsId, role.id, (permissions[role.id] as string[] | undefined) ?? []))
      );

      setHasChanges(false);
      void refetch();
    } catch (e) {
      logError(e, { component: "useWorkspacePermissionsPage" });
    } finally {
      setIsSaving(false);
    }
  };

  return {
    wsId,
    t,
    roles,
    matrix,
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

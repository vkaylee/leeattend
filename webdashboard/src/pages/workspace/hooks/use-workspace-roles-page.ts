import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useConfirm } from "@/hooks/use-confirm";
import { useWorkspaceRoles } from "@/hooks/use-workspace";

import type { WorkspaceRole } from "../../../lib/api/types";

export function useWorkspaceRolesPage() {
  const { t } = useTranslation();
  const { confirm, ConfirmationDialog } = useConfirm();
  const { roles, isLoading, deleteRole } = useWorkspaceRoles();
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<WorkspaceRole | null>(null);

  const handleCreate = () => {
    setEditingRole(null);
    setIsModalOpen(true);
  };

  const handleEdit = (role: WorkspaceRole) => {
    setEditingRole(role);
    setIsModalOpen(true);
    setActiveMenu(null);
  };

  const handleDelete = async (roleId: string) => {
    const isConfirmed = await confirm({
      title: t('rolesPage.deleteTitle', { defaultValue: 'Delete Role' }),
      message: t('rolesPage.confirmDelete', { defaultValue: "Are you sure you want to delete this role?" }),
      variant: 'danger',
    });
    if (isConfirmed) {
      await deleteRole(roleId);
    }
    setActiveMenu(null);
  };

  return {
    t,
    roles,
    isLoading,
    activeMenu,
    setActiveMenu,
    isModalOpen,
    setIsModalOpen,
    editingRole,
    handleCreate,
    handleEdit,
    handleDelete,
    ConfirmationDialog,
  };
}

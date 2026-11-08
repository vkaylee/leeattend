import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useConfirm } from "@/hooks/use-confirm";
import { useOrganizationRoles } from "@/hooks/use-organization";

import type { OrganizationRole } from "../../../lib/api/types";

export function useOrganizationRolesPage() {
  const { t } = useTranslation();
  const { confirm, ConfirmationDialog } = useConfirm();
  const { roles, isLoading, deleteRole } = useOrganizationRoles();
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<OrganizationRole | null>(null);

  const handleCreate = () => {
    setEditingRole(null);
    setIsModalOpen(true);
  };

  const handleEdit = (role: OrganizationRole) => {
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

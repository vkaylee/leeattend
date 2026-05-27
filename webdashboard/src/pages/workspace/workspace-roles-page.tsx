import { Plus } from "lucide-react";
import React from "react";

import PermissionGate from "@/components/common/permission-gate";
import RoleFormModal from "@/components/features/settings/role-form-modal";
import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

import { RoleCard } from "./components/role-card";
import { useWorkspaceRolesPage } from "./hooks/use-workspace-roles-page";

export default function WorkspaceRolesPage() {
  const {
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
  } = useWorkspaceRolesPage();

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          subtitle={t("rolesPage.subtitle", {
            defaultValue: "Define custom roles and manage access levels across the workspace",
          })}
          title={t("rolesPage.title", { defaultValue: "Role Management" })}
        />
        <PermissionGate permission="members:roles:manage">
          <Button
            className="flex items-center gap-2 bg-primary-600 hover:bg-primary-700 text-white px-5 py-2.5 rounded-2xl font-bold text-sm transition-all shadow-lg shadow-primary-200 dark:shadow-primary-900/20 hover:shadow-xl hover:-translate-y-0.5 active:translate-y-0"
            onClick={handleCreate}
            type="button"
          >
            <Plus size={18} />
            {t("rolesPage.createRole", { defaultValue: "Create Role" })}
          </Button>
        </PermissionGate>
      </div>

      {isLoading ? (
        <div className="flex justify-center p-20">
          <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
        </div>
      ) : (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
          {roles.map((role) => (
            <RoleCard
              activeMenu={activeMenu}
              key={role.id}
              onDelete={handleDelete}
              onEdit={handleEdit}
              role={role}
              setActiveMenu={setActiveMenu}
            />
          ))}
        </div>
      )}

      <RoleFormModal editingRole={editingRole} isOpen={isModalOpen} onClose={() => { setIsModalOpen(false); }} />
      <ConfirmationDialog />
    </div>
  );
}

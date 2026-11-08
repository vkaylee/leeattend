import { Plus, ShieldCheck } from "lucide-react";
import React from "react";

import PermissionGate from "@/components/common/permission-gate";
import RoleFormModal from "@/components/features/settings/role-form-modal";
import Button from "@/components/ui/button";
import { SettingsHeader } from "@/components/ui/settings/settings-header";

import { RoleCard } from "./components/role-card";
import { useOrganizationRolesPage } from "./hooks/use-organization-roles-page";

export default function OrganizationRolesPage() {
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
  } = useOrganizationRolesPage();

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <SettingsHeader
        actions={
          <PermissionGate permission="members:roles:manage">
            <Button
              className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold normal-case tracking-normal shadow-xs"
              onClick={handleCreate}
              type="button"
              variant="primary"
            >
              <Plus size={16} />
              {t("rolesPage.createRole", { defaultValue: "Create Role" })}
            </Button>
          </PermissionGate>
        }
        description={t("rolesPage.subtitle", {
          defaultValue: "Define custom roles and manage access levels across the workspace",
        })}
        icon={<ShieldCheck size={20} />}
        title={t("rolesPage.title", { defaultValue: "Role Management" })}
      />

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

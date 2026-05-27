import React from "react";
import { useTranslation } from "react-i18next";

import PageHeader from "@/components/ui/page-header";

import { AssignRoleModal } from "./system-roles/components/assign-role-modal";
import { SystemRoleCard } from "./system-roles/components/system-role-card";
import { useSystemRolesUI } from "./system-roles/hooks/use-system-roles-ui";

export default function SystemRolesPage() {
  const { t } = useTranslation(["common", "system_roles"]);
  const {
    roles,
    isLoading,
    isAssigning,
    isRevoking,
    isModalOpen,
    selectedRole,
    userIdInput,
    assignError,
    setUserIdInput,
    setIsModalOpen,
    handleOpenAssign,
    handleAssignSubmit,
    handleRevoke,
    ConfirmationDialog,
  } = useSystemRolesUI();

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          subtitle={t("system_roles.subtitle")}
          title={t("system_roles.title")}
        />
      </div>

      {isLoading ? (
        <div className="flex justify-center p-20">
          <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
        </div>
      ) : (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
          {roles.map((role) => (
            <SystemRoleCard
              isRevoking={isRevoking}
              key={role.id}
              onAssign={handleOpenAssign}
              onRevoke={handleRevoke}
              role={role}
            />
          ))}
        </div>
      )}

      <AssignRoleModal
        assignError={assignError}
        isAssigning={isAssigning}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        onSubmit={handleAssignSubmit}
        onUserIdChange={setUserIdInput}
        selectedRole={selectedRole}
        userIdInput={userIdInput}
      />
      <ConfirmationDialog />
    </div>
  );
}

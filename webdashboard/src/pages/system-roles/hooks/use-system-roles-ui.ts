import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useSystemRoles } from "@/hooks/use-system-roles";

export function useSystemRolesUI() {
  const { t } = useTranslation();
  const { roles, isLoading, assignRole, revokeRole, isAssigning, isRevoking } = useSystemRoles();
  const { confirm, ConfirmationDialog } = useConfirm();
  const toast = useToast();
  
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [selectedRole, setSelectedRole] = useState<string | null>(null);
  const [userIdInput, setUserIdInput] = useState("");
  const [assignError, setAssignError] = useState("");

  const handleOpenAssign = (roleName: string) => {
    setSelectedRole(roleName);
    setUserIdInput("");
    setAssignError("");
    setIsModalOpen(true);
  };

  const handleAssignSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!selectedRole || !userIdInput.trim()) return;
    try {
      setAssignError("");
      await assignRole({ userId: userIdInput.trim(), request: { role_name: selectedRole } });
      setIsModalOpen(false);
      toast.success(t("system_roles.messages.assign_success", { role: selectedRole }));
    } catch (error: unknown) {
      setAssignError(error instanceof Error ? error.message : t("system_roles.messages.assign_error"));
    }
  };

  const handleRevoke = async (userId: string, roleName: string) => {
    const isConfirmed = await confirm({
      title: t("system_roles.confirm.revoke_title"),
      message: t("system_roles.confirm.revoke_message"),
      variant: "danger"
    });
    if (isConfirmed) {
      try {
        await revokeRole({ userId, roleName });
        toast.success(t("system_roles.messages.revoke_success", { role: roleName }));
      } catch (error: unknown) {
        toast.error(error instanceof Error ? error.message : t("system_roles.messages.revoke_error"));
      }
    }
  };

  return {
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
  };
}

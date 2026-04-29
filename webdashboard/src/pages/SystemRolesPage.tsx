import React, { useState } from "react";

import { ShieldAlert, ShieldCheck, Shield, Users, Plus, Trash2, ShieldHalf } from "lucide-react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";
import PageHeader from "@/components/ui/PageHeader";

import { useConfirm } from "@/hooks/use-confirm";
import { useSystemRoles } from "@/hooks/use-system-roles";
import { cn } from "@/lib/utils";



const roleDescriptions: Record<string, string> = {
  platform_admin: "system_roles.descriptions.platform_admin",
  support: "system_roles.descriptions.support",
  billing_admin: "system_roles.descriptions.billing_admin",
};

const roleIcons: Record<string, any> = {
  platform_admin: ShieldAlert,
  support: ShieldCheck,
  billing_admin: ShieldHalf,
};

export default function SystemRolesPage() {
  const { t } = useTranslation();
  const { roles, isLoading, assignRole, revokeRole, isAssigning, isRevoking } = useSystemRoles();
  const { confirm, ConfirmationDialog } = useConfirm();
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

  const handleAssignSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedRole || !userIdInput.trim()) return;
    try {
      setAssignError("");
      await assignRole({ userId: userIdInput.trim(), request: { role_name: selectedRole } });
      setIsModalOpen(false);
    } catch (err: any) {
      setAssignError(err.message || t("system_roles.messages.assign_error"));
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
      } catch (err: any) {
        alert(err.message || t("system_roles.messages.revoke_error"));
      }
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          title={t("system_roles.title")}
          subtitle={t("system_roles.subtitle")}
        />
      </div>

      {isLoading ? (
        <div className="flex justify-center p-20">
          <div className="w-10 h-10 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin"></div>
        </div>
      ) : (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
          {roles.map((role) => {
            const Icon = roleIcons[role.name] || Shield;
            
            return (
              <div
                key={role.id}
                className="bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 p-6 shadow-sm hover:shadow-xl hover:shadow-slate-200/50 dark:hover:shadow-black/20 transition-all duration-300 group flex flex-col"
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-4">
                    <div className="w-14 h-14 rounded-2xl flex items-center justify-center border bg-indigo-50 text-indigo-600 dark:bg-indigo-500/10 dark:text-indigo-400 border-indigo-200/50 dark:border-indigo-500/20">
                      <Icon size={24} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200">
                          {role.name}
                        </h3>
                        <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-rose-100 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400">
                          {t("system_roles.system_label")}
                        </span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm font-medium text-slate-500 mt-1">
                        <Users size={14} />
                        <span>{t("system_roles.assignees_count", { count: role.assignees?.length || 0 })}</span>
                      </div>
                    </div>
                  </div>
                  
                  <button
                    onClick={() => handleOpenAssign(role.name)}
                    className="p-2 text-indigo-600 bg-indigo-50 hover:bg-indigo-100 dark:bg-indigo-500/10 dark:text-indigo-400 dark:hover:bg-indigo-500/20 rounded-xl transition-colors"
                    title={t("system_roles.assign_to_user")}
                  >
                    <Plus size={20} />
                  </button>
                </div>

                <p className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-6 flex-1">
                  {role.description || t(roleDescriptions[role.name] || "system_roles.descriptions.default")}
                </p>

                <div className="space-y-3 pt-4 border-t border-slate-100 dark:border-slate-800">
                  <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider">{t("system_roles.assigned_users")}</h4>
                  {role.assignees?.length === 0 ? (
                    <p className="text-sm text-slate-500 italic">{t("system_roles.no_users")}</p>
                  ) : (
                    <div className="space-y-2 max-h-[200px] overflow-y-auto pr-2 custom-scrollbar">
                      {role.assignees?.map((assignee) => (
                        <div key={assignee.user_id} className="flex items-center justify-between p-3 rounded-xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800">
                          <div className="flex flex-col">
                            <span className="text-sm font-bold text-slate-700 dark:text-slate-300 font-mono">
                              {assignee.user_id}
                            </span>
                            <span className="text-xs text-slate-500">
                              {t("system_roles.assigned_date", { date: new Date(assignee.assigned_at).toLocaleDateString() })}
                            </span>
                          </div>
                          <button
                            onClick={() => handleRevoke(assignee.user_id, role.name)}
                            disabled={isRevoking}
                            className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-500/10 rounded-lg transition-colors disabled:opacity-50"
                            title={t("system_roles.revoke_role")}
                          >
                            <Trash2 size={16} />
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={t("system_roles.modal.assign_title", { role: selectedRole })}
        subtitle={t("system_roles.grant_privileges")}
        maxWidth="md"
      >
        <form onSubmit={handleAssignSubmit} className="space-y-6">
          <div className="space-y-2">
            <label className="text-sm font-bold text-slate-700 dark:text-slate-300">
              {t("system_roles.user_id_label")}
            </label>
            <input
              type="text"
              value={userIdInput}
              onChange={(e) => setUserIdInput(e.target.value)}
              placeholder="e.g. 123e4567-e89b-12d3-a456-426614174000"
              className="w-full px-4 py-3 rounded-xl bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all font-mono text-sm"
              required
            />
            {assignError && (
              <p className="text-sm text-rose-500 font-medium mt-1">{assignError}</p>
            )}
          </div>
          
          <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
            <button
              type="button"
              onClick={() => setIsModalOpen(false)}
              className="px-5 py-2.5 text-sm font-bold text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors"
            >
              {t("common.cancel")}
            </button>
            <button
              type="submit"
              disabled={isAssigning || !userIdInput.trim()}
              className="px-5 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-xl transition-all disabled:opacity-50 shadow-md shadow-indigo-200 dark:shadow-none"
            >
              {isAssigning ? t("system_roles.modal.assigning") : t("system_roles.modal.submit_btn")}
            </button>
          </div>
        </form>
      </Modal>
      <ConfirmationDialog />
    </div>
  );
}

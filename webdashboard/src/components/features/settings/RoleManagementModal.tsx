import { Shield, Plus, Trash2, Clock, UserCheck } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { useConfirm } from "@/hooks/use-confirm";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMemberRoles, useWorkspaceRoles } from "@/hooks/use-workspace";

interface RoleManagementModalProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string | null;
}

export default function RoleManagementModal({
  userId,
  isOpen,
  onClose,
}: RoleManagementModalProps) {
  const { t } = useTranslation();
  const { formatDateTime } = useDateTimeFormatter();
  const { roles: memberRoles, isLoading, updatePrimaryRole, assignRole, revokeRole } = useMemberRoles(userId || "", undefined);
  const { roles: workspaceRoles } = useWorkspaceRoles();
  const { confirm, ConfirmationDialog } = useConfirm();

  const [newRoleId, setNewRoleId] = useState("");
  const [expiresAt, setExpiresAt] = useState("");

  const primaryRoleId = memberRoles?.primary || "";
  const extraRoles = memberRoles?.extra || [];

  const getRoleName = (roleId: string) => {
    return workspaceRoles.find(r => r.id === roleId)?.name || roleId;
  };

  const roleOptions = workspaceRoles.map(r => ({
    value: r.id,
    label: r.name.toUpperCase(),
    description: r.description || undefined,
    icon: <Shield className={`w-4 h-4 ${r.is_system ? 'text-primary-600 dark:text-primary-400' : 'text-slate-500 dark:text-slate-400'}`} />,
  }));

  const availableExtraRoleOptions = roleOptions.filter(opt =>
    opt.value !== primaryRoleId && !extraRoles.some(er => er.role_id === opt.value)
  );

  async function handleUpdatePrimary(roleId: string) {
    if (roleId === primaryRoleId) return;
    const roleName = getRoleName(roleId);
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("members.roles.confirm_change_primary", { role: roleName.toUpperCase() }),
      variant: "danger"
    });
    if (!ok) return;
    try { await updatePrimaryRole(roleId); } catch (e: unknown) { console.error(e); }
 
  }

  async function handleAssign(e: React.FormEvent) {
    e.preventDefault(); if (!expiresAt || !userId || !newRoleId) return;
    try {
      const isoExpiresAt = new Date(expiresAt).toISOString();
      await assignRole({ role: newRoleId, expiresAt: isoExpiresAt });
      setExpiresAt("");
      setNewRoleId("");
    } catch (e) { console.error(e); }
  }

  async function handleRevoke(roleId: string) {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("members.roles.confirm_revoke"),
      variant: "danger"
    });
    if (!ok) return;
    try { await revokeRole(roleId); } catch (e) { console.error(e); }
  }

  if (!isOpen) return null;

  return (
    <Modal
      icon={<UserCheck size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("members.roles.modal_subtitle")}
      title={t("members.roles.modal_title")}
    >
      <div className="space-y-10">
          {isLoading ? (
            <div className="py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</div>
          ) : (
            <div className="space-y-10 animate-in fade-in duration-700">
              <div className="space-y-4 px-1">
                <label className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400 ml-1" htmlFor="primary-role-select">
                  {t("members.roles.primary_role_label").toUpperCase()}
                </label>
                <Combobox
                  className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-16 rounded-[1.5rem] text-sm shadow-inner"
                  id="primary-role-select"
                  onChange={handleUpdatePrimary}
                  options={roleOptions}
                  value={primaryRoleId}
                />
              </div>

              <div className="space-y-5 px-1">
                <label className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400 ml-1">
                  {t("members.roles.extra_roles_label").toUpperCase()}
                </label>

                <div className="space-y-4">
                  {extraRoles.map((extraRole) => (
                    <div className="flex items-center justify-between p-6 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2rem] hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden shadow-inner" key={extraRole.id}>
                      <div className="space-y-2">
                        <p className="font-bold text-slate-700 dark:text-slate-200 text-base uppercase tracking-tight leading-none">{getRoleName(extraRole.role_id)}</p>
                        <div className="flex items-center gap-2 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70">
                          <Clock className="text-primary-400" size={12} /> <span>{t("members.roles.expires_at").toUpperCase()}: {formatDateTime(extraRole.expires_at)}</span>
                        </div>
                      </div>
                      <button className="p-3 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0 active:scale-90" onClick={() => handleRevoke(extraRole.id)}><Trash2 size={18} /></button>
                    </div>
                  ))}

                  {extraRoles.length === 0 && (
                    <div className="text-center py-12 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
                      <Shield className="text-slate-200 dark:text-slate-700 mb-4" size={32} />
                      <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest opacity-60 italic">{t("members.roles.no_extra_roles")}</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          <div className="pt-10 border-t border-slate-50 dark:border-slate-800 space-y-8">
            <h3 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-3 px-1">
              <Plus className="text-primary-500" size={16} /> {t("members.roles.assign_new_title").toUpperCase()}
            </h3>
            <form className="grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-bottom-4 duration-700" onSubmit={handleAssign}>
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="extra-role-select">{t("members.invite.role_label")}</label>
                <Combobox
                  className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14 rounded-2xl"
                  id="extra-role-select"
                  onChange={setNewRoleId}
                  options={availableExtraRoleOptions}
                  value={newRoleId}
                />
              </div>
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="expires-at-input">{t("members.roles.expiry_date_label")}</label>
                <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" id="expires-at-input" onChange={(e) => { setExpiresAt(e.target.value); }} required type="datetime-local" value={expiresAt} />
              </div>
              <div className="md:col-span-2 pt-2">
                <button className="w-full bg-slate-900 text-white py-5 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] hover:bg-primary-700 transition-all shadow-2xl shadow-slate-400 dark:shadow-black/40 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed" disabled={!newRoleId} type="submit">{t("members.roles.assign_btn").toUpperCase()}</button>
              </div>
            </form>
          </div>
      </div>
      <ConfirmationDialog />
    </Modal>
  );
}

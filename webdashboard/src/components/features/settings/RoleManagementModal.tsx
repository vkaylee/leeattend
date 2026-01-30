import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Shield, Plus, Trash2, Clock, UserCheck } from "lucide-react";
import { useMemberRoles } from "@/hooks/use-workspace";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";

interface RoleManagementModalProps {
  userId: string | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function RoleManagementModal({
  userId,
  isOpen,
  onClose,
}: RoleManagementModalProps) {
  const { t } = useTranslation();
  const { formatDateTime } = useDateTimeFormatter();
  const { roles, isLoading, updatePrimaryRole, assignRole, revokeRole } = useMemberRoles(userId || "", undefined);
  const [newRole, setNewRole] = useState("member");
  const [expiresAt, setExpiresAt] = useState("");

  const primaryRole = roles?.primary || "";
  const extraRoles = roles?.extra || [];

  async function handleUpdatePrimary(role: string) {
    if (role === primaryRole) return;
    if (!confirm(t("members.roles.confirm_change_primary", { role: role.toUpperCase() }))) return;
    try { await updatePrimaryRole(role); } catch (e: any) { console.error(e); }
  }

  async function handleAssign(e: React.FormEvent) {
    e.preventDefault(); if (!expiresAt || !userId) return;
    try {
      const isoExpiresAt = new Date(expiresAt).toISOString();
      await assignRole({ role: newRole, expiresAt: isoExpiresAt });
      setExpiresAt("");
    } catch (e) { console.error(e); }
  }

  async function handleRevoke(roleId: string) {
    if (!confirm(t("members.roles.confirm_revoke"))) return;
    try { await revokeRole(roleId); } catch (e) { console.error(e); }
  }

  if (!isOpen) return null;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-content max-w-lg shadow-premium-2xl" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between p-10 border-b border-slate-50 bg-slate-50/30">
          <div className="flex items-center gap-5">
            <div className="bg-primary-50 p-4 rounded-2xl text-primary-600 shadow-inner border border-primary-100/50">
              <UserCheck size={32} />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-slate-700 tracking-tighter uppercase leading-none">
                {t("members.roles.modal_title")}
              </h2>
              <p className="text-[10px] text-slate-500 font-bold uppercase tracking-[0.2em] mt-2 opacity-60">{t("members.roles.modal_subtitle")}</p>
            </div>
          </div>
          <button onClick={onClose} className="p-3 bg-white hover:bg-slate-50 rounded-xl transition-all shadow-md border border-slate-100 text-slate-300 hover:text-rose-600 active:scale-90"><X size={24} /></button>
        </div>

        <div className="p-10 space-y-10">
          {isLoading ? (
            <div className="py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</div>
          ) : (
            <div className="space-y-10 animate-in fade-in duration-700">
              <div className="space-y-4 px-1">
                <label htmlFor="primary-role-select" className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 ml-1">
                  {t("members.roles.primary_role_label").toUpperCase()}
                </label>
                <Combobox
                  id="primary-role-select"
                  options={[
                    { value: "owner", label: t("members.roles.owner").toUpperCase(), description: t("members.roles.owner_desc"), icon: <Shield className="w-4 h-4 text-primary-600" /> },
                    { value: "admin", label: t("members.roles.admin").toUpperCase(), description: t("members.roles.admin_desc"), icon: <Shield className="w-4 h-4 text-primary-600" /> },
                    { value: "member", label: t("members.roles.member").toUpperCase(), description: t("members.roles.member_desc"), icon: <Shield className="w-4 h-4 text-slate-500" /> },
                  ]}
                  value={primaryRole}
                  onChange={handleUpdatePrimary}
                  className="bg-slate-50 border-slate-100 h-16 rounded-[1.5rem] text-sm shadow-inner"
                />
              </div>

              <div className="space-y-5 px-1">
                <label className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 ml-1">
                  {t("members.roles.extra_roles_label").toUpperCase()}
                </label>

                <div className="space-y-4">
                  {extraRoles.map((role) => (
                    <div key={role.id} className="flex items-center justify-between p-6 bg-slate-50 border border-slate-100 rounded-[2rem] hover:bg-white hover:shadow-xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden shadow-inner">
                      <div className="space-y-2">
                        <p className="font-bold text-slate-700 text-base uppercase tracking-tight leading-none">{role.role}</p>
                        <div className="flex items-center gap-2 text-[10px] text-slate-500 font-bold uppercase tracking-widest opacity-70">
                          <Clock size={12} className="text-primary-400" /> <span>{t("members.roles.expires_at").toUpperCase()}: {formatDateTime(role.expires_at)}</span>
                        </div>
                      </div>
                      <button onClick={() => handleRevoke(role.id)} className="p-3 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0 active:scale-90"><Trash2 size={18} /></button>
                    </div>
                  ))}

                  {extraRoles.length === 0 && (
                    <div className="text-center py-12 border-2 border-dashed border-slate-100 rounded-[2.5rem] bg-slate-50/30 flex flex-col items-center">
                      <Shield size={32} className="text-slate-200 mb-4" />
                      <p className="text-[10px] font-bold text-slate-300 uppercase tracking-widest opacity-60 italic">{t("members.roles.no_extra_roles")}</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          <div className="pt-10 border-t border-slate-50 space-y-8">
            <h3 className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.3em] flex items-center gap-3 px-1">
              <Plus size={16} className="text-primary-500" /> {t("members.roles.assign_new_title").toUpperCase()}
            </h3>
            <form onSubmit={handleAssign} className="grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-bottom-4 duration-700">
              <div className="space-y-3 px-1">
                <label htmlFor="extra-role-select" className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("members.invite.role_label")}</label>
                <Combobox id="extra-role-select" options={[{ value: "member", label: t("members.roles.member").toUpperCase(), icon: <Shield className="w-4 h-4" /> }, { value: "admin", label: t("members.roles.admin").toUpperCase(), icon: <Shield className="w-4 h-4 text-primary-600" /> }]} value={newRole} onChange={setNewRole} className="bg-slate-50 border-slate-100 h-14 rounded-2xl" />
              </div>
              <div className="space-y-3 px-1">
                <label htmlFor="expires-at-input" className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("members.roles.expiry_date_label")}</label>
                <input id="expires-at-input" required type="datetime-local" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} />
              </div>
              <div className="md:col-span-2 pt-2">
                <button type="submit" className="w-full bg-slate-900 text-white py-5 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] hover:bg-primary-700 transition-all shadow-2xl shadow-slate-400 active:scale-95">{t("members.roles.assign_btn").toUpperCase()}</button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

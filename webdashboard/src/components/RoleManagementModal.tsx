import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Shield, Plus, Trash2, Clock, UserCheck } from "lucide-react";
import { useMemberRoles } from "@/hooks/use-workspace";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "./ui/Combobox";

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
  const { roles, isLoading, updatePrimaryRole, assignRole, revokeRole } =
    useMemberRoles(userId || "", undefined);
  const [newRole, setNewRole] = useState("member");
  const [expiresAt, setExpiresAt] = useState("");

  const primaryRole = roles?.primary || "";
  const extraRoles = roles?.extra || [];

  async function handleUpdatePrimary(role: string) {
    if (role === primaryRole) return;
    if (!confirm(t("members.roles.confirm_change_primary", { role: role.toUpperCase() }))) return;
    try { await updatePrimaryRole(role); } catch (e: any) { }
  }

  async function handleAssign(e: React.FormEvent) {
    e.preventDefault();
    if (!expiresAt || !userId) return;
    try {
      const isoExpiresAt = new Date(expiresAt).toISOString();
      await assignRole({ role: newRole, expiresAt: isoExpiresAt });
      setExpiresAt("");
    } catch (e) { }
  }

  async function handleRevoke(roleId: string) {
    if (!confirm(t("members.roles.confirm_revoke"))) return;
    try { await revokeRole(roleId); } catch (e) { }
  }

  if (!isOpen) return null;

  return (
    <div className="modal-backdrop">
      <div className="modal-content max-w-lg shadow-premium-2xl">
        <div className="flex items-center justify-between p-10 border-b border-slate-50 bg-slate-50/30">
          <div className="flex items-center gap-5">
            <div className="bg-primary-50 p-4 rounded-2xl text-primary-600 shadow-inner border border-primary-100/50">
              <UserCheck size={24} />
            </div>
            <div>
              <h2 className="text-xl font-black text-slate-900 tracking-tight uppercase">
                {t("members.roles.modal_title")}
              </h2>
              <p className="text-[10px] text-slate-400 font-black uppercase tracking-widest mt-1 opacity-70">{t("members.roles.modal_subtitle")}</p>
            </div>
          </div>
          <button onClick={onClose} className="p-3 bg-white hover:bg-slate-50 rounded-xl transition-all shadow-md border border-slate-100 text-slate-400 hover:text-rose-600 active:scale-90"><X size={20} /></button>
        </div>

        <div className="p-10 space-y-10">
          {isLoading ? (
            <div className="py-16 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>
          ) : (
            <div className="space-y-10 animate-in fade-in duration-500">
              {/* Primary Role */}
              <div className="space-y-3">
                <label htmlFor="primary-role-select" className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1">
                  {t("members.roles.primary_role_label")}
                </label>
                <Combobox
                  id="primary-role-select"
                  options={[
                    { value: "owner", label: t("members.roles.owner"), description: t("members.roles.owner_desc"), icon: <Shield className="w-4 h-4 text-primary-600" /> },
                    { value: "admin", label: t("members.roles.admin"), description: t("members.roles.admin_desc"), icon: <Shield className="w-4 h-4 text-primary-600" /> },
                    { value: "member", label: t("members.roles.member"), description: t("members.roles.member_desc"), icon: <Shield className="w-4 h-4 text-slate-400" /> },
                  ]}
                  value={primaryRole}
                  onChange={handleUpdatePrimary}
                  className="bg-slate-50 border-slate-100 h-16 rounded-[1.25rem] text-sm"
                />
              </div>

              {/* Extra Roles */}
              <div className="space-y-4">
                <label className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1">
                  {t("members.roles.extra_roles_label")}
                </label>

                <div className="space-y-3">
                  {extraRoles.map((role) => (
                    <div key={role.id} className="flex items-center justify-between p-6 bg-slate-50 border border-slate-100 rounded-[2rem] hover:bg-white hover:shadow-xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden shadow-inner">
                      <div className="space-y-1">
                        <p className="font-black text-slate-800 text-sm uppercase tracking-tight">{role.role}</p>
                        <div className="flex items-center gap-2 text-[10px] text-slate-400 font-bold uppercase tracking-widest opacity-70">
                          <Clock size={12} /> <span>{t("members.roles.expires_at")}: {formatDateTime(role.expires_at)}</span>
                        </div>
                      </div>
                      <button onClick={() => handleRevoke(role.id)} className="p-2.5 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button>
                    </div>
                  ))}

                  {extraRoles.length === 0 && (
                    <div className="text-center py-10 border-2 border-dashed border-slate-100 rounded-[2rem] bg-slate-50/30 flex flex-col items-center">
                      <Shield size={24} className="text-slate-200 mb-2" />
                      <p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("members.roles.no_extra_roles")}</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Assign Form */}
          <div className="pt-10 border-t border-slate-100 space-y-6">
            <h3 className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] flex items-center gap-2">
              <Plus size={14} className="text-primary-500" /> {t("members.roles.assign_new_title")}
            </h3>
            <form onSubmit={handleAssign} className="grid grid-cols-1 md:grid-cols-2 gap-6 animate-in slide-in-from-bottom-2">
              <div className="space-y-2">
                <label htmlFor="extra-role-select" className="text-[10px] font-black text-slate-500 uppercase tracking-widest ml-1">{t("members.invite.role_label")}</label>
                <Combobox id="extra-role-select" options={[{ value: "member", label: t("members.roles.member"), icon: <Shield className="w-4 h-4" /> }, { value: "admin", label: t("members.roles.admin"), icon: <Shield className="w-4 h-4 text-primary-600" /> }]} value={newRole} onChange={setNewRole} className="bg-slate-50 border-slate-100 h-[52px]" />
              </div>
              <div className="space-y-2">
                <label htmlFor="expires-at-input" className="text-[10px] font-black text-slate-500 uppercase tracking-widest ml-1">{t("members.roles.expiry_date_label")}</label>
                <input id="expires-at-input" required type="datetime-local" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-3.5 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} />
              </div>
              <div className="md:col-span-2 pt-2">
                <button type="submit" className="w-full bg-slate-900 text-white py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("members.roles.assign_btn").toUpperCase()}</button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

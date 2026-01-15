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
  const { formatDate, formatTime, formatDateTime } = useDateTimeFormatter();
  const { roles, isLoading, updatePrimaryRole, assignRole, revokeRole } =
    useMemberRoles(userId || "", undefined);
  const [newRole, setNewRole] = useState("member");
  const [expiresAt, setExpiresAt] = useState("");

  const primaryRole = roles?.primary || "";
  const extraRoles = roles?.extra || [];

  async function handleUpdatePrimary(role: string) {
    if (role === primaryRole) return;
    if (
      !confirm(t("members.roles.confirm_change_primary", { role: role.toUpperCase() }))
    )
      return;

    try {
      await updatePrimaryRole(role);
    } catch (e: any) {
      alert(e.message || t("members.roles.messages.update_primary_error"));
    }
  }

  async function handleAssign(e: React.FormEvent) {
    e.preventDefault();
    if (!expiresAt || !userId) return;

    try {
      const isoExpiresAt = new Date(expiresAt).toISOString();
      await assignRole({ role: newRole, expiresAt: isoExpiresAt });
      setExpiresAt("");
    } catch (e) {
      alert(t("members.roles.messages.assign_error"));
    }
  }

  async function handleRevoke(roleId: string) {
    if (!confirm(t("members.roles.confirm_revoke"))) return;
    try {
      await revokeRole(roleId);
    } catch (e) {
      alert(t("members.roles.messages.revoke_error"));
    }
  }

  if (!isOpen) return null;

  return (
    <div className="modal-backdrop">
      <div className="modal-content max-w-lg">
        <div className="flex items-center justify-between p-8 border-b border-slate-50">
          <div className="flex items-center gap-4">
            <div className="bg-primary-50 p-3 rounded-2xl text-primary-600">
              <UserCheck className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-slate-900 tracking-tight">
                {t("members.roles.modal_title")}
              </h2>
              <p className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-0.5">{t("members.roles.modal_subtitle")}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-100 rounded-full transition-colors group"
          >
            <X className="w-5 h-5 text-slate-300 group-hover:text-slate-600" />
          </button>
        </div>

        <div className="p-8 space-y-8">
          {isLoading ? (
            <div className="py-12 text-center text-slate-400">{t("common.loading")}</div>
          ) : (
            <>
              {/* Primary Role */}
              <div className="space-y-3">
                <label 
                  htmlFor="primary-role-select"
                  className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1"
                >
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
                  className="bg-slate-50 border-slate-100 h-14"
                />
              </div>

              {/* Extra Roles */}
              <div className="space-y-4">
                <label className="text-[10px] font-black uppercase tracking-widest text-slate-400 ml-1">
                  {t("members.roles.extra_roles_label")}
                </label>

                <div className="space-y-3">
                  {extraRoles.map((role) => (
                    <div
                      key={role.id}
                      className="flex items-center justify-between p-4 bg-white rounded-2xl border border-slate-100 group hover:border-blue-200 transition-all"
                    >
                      <div className="space-y-1">
                        <p className="font-bold text-slate-700 capitalize">
                          {role.role}
                        </p>
                        <div className="flex items-center gap-2 text-xs text-slate-400">
                          <Clock className="w-3 h-3" />
                          <span>
                            {t("members.roles.expires_at")}:{" "}
                            {formatDateTime(role.expires_at)}
                          </span>
                        </div>
                      </div>
                      <button
                        onClick={() => handleRevoke(role.id)}
                        title={t("common.remove")}
                        className="p-2 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all opacity-0 group-hover:opacity-100"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}

                  {extraRoles.length === 0 && (
                    <div className="text-center py-4 border-2 border-dashed border-slate-100 rounded-2xl text-slate-400 text-sm">
                      {t("members.roles.no_extra_roles")}
                    </div>
                  )}
                </div>
              </div>
            </>
          )}

          {/* Assign Form */}
          <div className="pt-6 border-t border-slate-50">
            <h3 className="text-sm font-bold text-slate-800 mb-4 flex items-center gap-2">
              <Plus className="w-4 h-4 text-blue-500" />
              {t("members.roles.assign_new_title")}
            </h3>
            <form
              onSubmit={handleAssign}
              className="grid grid-cols-1 md:grid-cols-2 gap-4"
            >
              <div className="space-y-2">
                <label 
                  htmlFor="extra-role-select"
                  className="text-[10px] font-bold text-slate-400 uppercase tracking-widest ml-1"
                >
                  {t("members.invite.role_label")}
                </label>
                <Combobox
                  id="extra-role-select"
                  options={[
                    { value: "member", label: t("members.roles.member"), icon: <Shield className="w-4 h-4" /> },
                    { value: "admin", label: t("members.roles.admin"), icon: <Shield className="w-4 h-4 text-primary-600" /> },
                  ]}
                  value={newRole}
                  onChange={setNewRole}
                  className="bg-slate-50 border-slate-200"
                />
              </div>
              <div className="space-y-2">
                <label
                  htmlFor="expires-at-input"
                  className="text-[10px] font-bold text-slate-400 uppercase tracking-widest ml-1"
                >
                  {t("members.roles.expiry_date_label")}
                </label>
                <div className="relative">
                  <input
                    id="expires-at-input"
                    required
                    type="datetime-local"
                    className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2 focus:ring-2 focus:ring-primary-500 outline-none transition-all text-sm"
                    value={expiresAt}
                    onChange={(e) => setExpiresAt(e.target.value)}
                  />
                </div>
              </div>
              <div className="md:col-span-2 pt-2">
                <button
                  type="submit"
                  className="w-full bg-slate-900 text-white py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200 text-sm"
                >
                  {t("members.roles.assign_btn")}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

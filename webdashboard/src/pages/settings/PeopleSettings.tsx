import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Users, UserPlus, Mail, Settings2, Trash2 } from "lucide-react";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import { useToast } from "@/components/Toast";
import { Combobox } from "@/components/ui/Combobox";
import RoleManagementModal from "@/components/RoleManagementModal";
import DepartmentsPage from "@/pages/DepartmentsPage";

export default function PeopleSettings() {
  const { t } = useTranslation();
  const { members, isLoading, addMember, isAdding } = useWorkspaceMembers();
  const toast = useToast();
  const [newMemberEmail, setNewMemberEmail] = useState("");
  const [newMemberRole, setNewMemberRole] = useState<"admin" | "member">("member");
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);

  async function handleInvite(e: React.FormEvent) {
    e.preventDefault();
    try {
      await addMember({ email: newMemberEmail, role: newMemberRole });
      toast.success(`${t("common.workspace_settings.invite_member")} ${newMemberEmail}`);
      setNewMemberEmail("");
    } catch (e) {
      toast.error(t("common.error"));
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
          {t("common.settings.categories.people")}
        </h1>
        <p className="text-slate-500 text-sm mt-1">
          {t("common.settings.categories.people_desc")}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-8">
        {/* Invite Section */}
        <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
          <h2 className="text-lg font-bold mb-6 flex items-center gap-2 text-slate-800">
            <UserPlus className="w-5 h-5 text-blue-500" />
            {t("common.workspace_settings.invite_member")}
          </h2>
          <form onSubmit={handleInvite} className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-[240px] space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                {t("common.email")}
              </label>
              <div className="relative">
                <input
                  required
                  type="email"
                  placeholder="dong-nghiep@congty.com"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all pr-10"
                  value={newMemberEmail}
                  onChange={(e) => setNewMemberEmail(e.target.value)}
                />
                <Mail className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
              </div>
            </div>
            <div className="w-56 space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("common.role")}
              </label>
              <Combobox
                options={[
                  { value: "member", label: t("common.members.roles.member"), description: t("common.members.roles.member_desc") },
                  { value: "admin", label: t("common.members.roles.admin"), description: t("common.members.roles.admin_desc") },
                ]}
                value={newMemberRole}
                onChange={(val) => setNewMemberRole(val as any)}
                className="bg-slate-50 border-slate-200"
              />
            </div>
            <div className="flex items-end">
              <button
                type="submit"
                disabled={isAdding}
                className="bg-primary-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 h-[50px] disabled:opacity-50"
              >
                {isAdding ? t("common.inviting") : t("common.workspace_settings.invite_member")}
              </button>
            </div>
          </form>
        </div>

        {/* Members List */}
        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
          <div className="p-6 border-b border-slate-50 flex items-center justify-between">
            <h2 className="text-lg font-bold flex items-center gap-2 text-slate-800">
              <Users className="w-5 h-5 text-slate-400" />
              {t("common.workspace_settings.members_list")}
            </h2>
            <div className="flex items-center gap-2 px-3 py-1 bg-slate-100 rounded-lg">
              <span className="text-xs font-bold text-slate-500">
                {members.length} {t("members.title").toUpperCase()}
              </span>
            </div>
          </div>

          <div className="divide-y divide-slate-50">
            {isLoading ? (
              <div className="p-12 text-center text-slate-400 font-medium italic">
                {t("common.loading")}
              </div>
            ) : (
              members.map((member) => (
                <div
                  key={member.user_id}
                  className="p-6 flex items-center justify-between hover:bg-slate-50/50 transition-colors group"
                >
                  <div className="flex items-center gap-4">
                    <div className={`w-12 h-12 rounded-2xl flex items-center justify-center font-bold shadow-sm transition-all ${
                      member.role === 'owner' ? 'bg-primary-600 text-white shadow-primary-100' : 
                      member.role === 'admin' ? 'bg-primary-600 text-white shadow-primary-100' : 'bg-slate-100 text-slate-500'
                    }`}>
                      {member.full_name ? member.full_name[0] : <Users className="w-5 h-5" />}
                    </div>
                    <div>
                      <p className="font-bold text-slate-900 flex items-center gap-2">
                        {member.full_name || member.email}
                        {member.role === "owner" && <span className="text-[10px] bg-primary-50 text-primary-600 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter border border-primary-100">{t("common.workspace_settings.founder")}</span>}
                      </p>
                      <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">
                        {member.role === 'admin' ? t("common.members.roles.admin") : member.role === 'owner' ? t("common.members.roles.owner") : t("common.members.roles.member")}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => {
                        setSelectedUserId(member.user_id);
                        setIsRoleModalOpen(true);
                      }}
                      className="p-2.5 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all"
                      title={t("common.members.table.manage_roles")}
                    >
                      <Settings2 className="w-5 h-5" />
                    </button>

                    {member.role !== "owner" && (
                      <button className="p-2.5 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all" title={t("common.members.table.remove_member")}>
                        <Trash2 className="w-5 h-5" />
                      </button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Departments Section */}
        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
           <div className="p-1">
              <DepartmentsPage />
           </div>
        </div>
      </div>

      <RoleManagementModal
        userId={selectedUserId}
        isOpen={isRoleModalOpen}
        onClose={() => setIsRoleModalOpen(false)}
      />
    </div>
  );
}
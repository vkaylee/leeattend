import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Users, UserPlus, Mail, Settings2, Trash2, Building2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useWorkspaceMembers, useWorkspaceRoles } from "@/hooks/use-workspace";
import { useToast } from "@/components/ui/Toast";
import { Combobox } from "@/components/ui/Combobox";
import Button from "@/components/ui/Button";
import StatusBadge from "@/components/ui/StatusBadge";
import { LABEL_BASE } from "@/components/ui/FormField";
import RoleManagementModal from "@/components/features/settings/RoleManagementModal";
import DepartmentsPage from "@/pages/DepartmentsPage";

type TabType = "members" | "departments";

export default function PeopleSettings() {

  const { t } = useTranslation();

  const { members, isLoading: isMembersLoading, addMember, isAdding } = useWorkspaceMembers();
  const { roles, isLoading: isRolesLoading } = useWorkspaceRoles();
  const isLoading = isMembersLoading || isRolesLoading;

  const toast = useToast();

  const [activeTab, setActiveTab] = useState<TabType>("members");

  const [newMemberEmail, setNewMemberEmail] = useState("");

  const [newMemberRoleId, setNewMemberRoleId] = useState("");

  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);

  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);



  async function handleInvite(e: React.FormEvent) {

    e.preventDefault();

    try {

      const roleId = newMemberRoleId || roles[0]?.id;
      if (!roleId) throw new Error("No role selected");
      await addMember({ email: newMemberEmail, roleId });

      toast.success(`${t("common.workspace_settings.invite_member")} ${newMemberEmail}`);

      setNewMemberEmail("");

    } catch (e) { console.error(e); }

  }



  return (

    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">

        <div className="flex items-center gap-4">

          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Users size={24} /></div>

          <div>

            <h1 data-testid="people-settings-title" className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">{t("common.settings.categories.people")}</h1>

            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.people_desc")}</p>

          </div>

        </div>

      </div>



      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">

        <button data-testid="tab-members" onClick={() => setActiveTab("members")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'members' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Users size={14} /> {t("common.settings.tabs.members")}</button>

        <button data-testid="tab-departments" onClick={() => setActiveTab("departments")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'departments' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Building2 size={14} /> {t("common.settings.tabs.departments")}</button>

      </div>



      <div className="animate-in slide-in-from-bottom-4 duration-500">

        {activeTab === "members" && (

          <div className="space-y-8">

            <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10">

              <div className="flex items-center gap-3 border-b border-slate-50 dark:border-slate-800 pb-6"><UserPlus className="w-6 h-6 text-primary-500" /><h2 data-testid="invite-member-heading" className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("common.workspace_settings.invite_member")}</h2></div>

              <form onSubmit={handleInvite} className="flex flex-wrap gap-6 items-end">

                <div className="flex-1 min-w-[280px] space-y-3">

                  <label className={LABEL_BASE}>{t("common.email")}</label>

                  <div className="relative group">

                    <input required type="email" placeholder="dong-nghiep@congty.com" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-14 font-bold uppercase tracking-tight" value={newMemberEmail} onChange={(e) => setNewMemberEmail(e.target.value)} />

                    <Mail className="absolute right-5 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />

                  </div>

                </div>

                <div className="w-64 space-y-3">

                  <label className={LABEL_BASE}>{t("common.role")}</label>

                  <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold uppercase tracking-tight h-14" value={newMemberRoleId || (roles[0]?.id ?? "")} onChange={(e) => setNewMemberRoleId(e.target.value)}>
                    {roles.map(r => (
                      <option key={r.id} value={r.id}>{r.name.toUpperCase()}</option>
                    ))}
                  </select>

                </div>

                <Button data-testid="invite-member-submit-btn" type="submit" loading={isAdding} className="h-[56px]">{isAdding ? t("common.inviting").toUpperCase() : t("common.workspace_settings.invite_member").toUpperCase()}</Button>

              </form>

            </div>



            <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm">

              <div className="p-8 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30 px-10">

                <div className="flex items-center gap-3"><Users className="w-6 h-6 text-slate-500 dark:text-slate-400" /><h2 data-testid="members-list-heading" className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("common.workspace_settings.members_list")}</h2></div>

                <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-4 py-1.5 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm uppercase tracking-widest">{members.length} {t("members.title").toUpperCase()}</span>

              </div>



              <div className="divide-y divide-slate-50 dark:divide-slate-800">

                {isLoading ? (

                  <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>

                ) : (

                  members.map((member) => (

                    <div key={member.user_id} className="p-8 px-10 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">

                      <div className="flex items-center gap-5">

                        <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center font-bold text-lg shadow-inner border transition-all uppercase", 
                          (() => {
                            const r = roles.find(role => role.id === member.role_id);
                            const isOwnerOrAdmin = r && (r.name.toLowerCase() === 'owner' || r.name.toLowerCase() === 'admin');
                            return isOwnerOrAdmin ? 'bg-primary-600 text-white border-primary-500 shadow-primary-100' : 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800';
                          })()
                        )}>{member.full_name ? member.full_name[0] : <Users size={20} />}</div>

                        <div>

                          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight flex items-center gap-3">

                            {member.full_name || member.email}

                            {roles.find(r => r.id === member.role_id)?.name.toLowerCase() === "owner" && <StatusBadge variant="warning">{t("common.workspace_settings.founder")}</StatusBadge>}
                          </div>
                          <p className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400 mt-1 opacity-70">
                            {roles.find(r => r.id === member.role_id)?.name || "Member"}
                          </p>

                        </div>

                      </div>



                      <div className="flex items-center gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">

                        <button onClick={() => { setSelectedUserId(member.user_id); setIsRoleModalOpen(true); }} className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.members.table.manage_roles")}><Settings2 size={18} /></button>

                        {roles.find(r => r.id === member.role_id)?.name.toLowerCase() !== "owner" && (

                          <button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.members.table.remove_member")}><Trash2 size={18} /></button>

                        )}

                      </div>

                    </div>

                  ))

                )}

              </div>

            </div>

          </div>

        )}



        {activeTab === "departments" && (

          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><DepartmentsPage /></div>

        )}

      </div>



      <RoleManagementModal userId={selectedUserId} isOpen={isRoleModalOpen} onClose={() => setIsRoleModalOpen(false)} />

    </div>

  );

}

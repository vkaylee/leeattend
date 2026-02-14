import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  MoreVertical,
  Shield,
  Trash2,
  UserCog,
  Mail,
  User as UserIcon,
} from "lucide-react";
import { WorkspaceMember } from "@/lib/api";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import RoleManagementModal from "@/components/features/settings/RoleManagementModal";
import { cn } from "@/lib/utils";

export default function MemberTable() {
  const { t } = useTranslation();
  const { members, isLoading, removeMember } = useWorkspaceMembers();
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);

  const handleManageRoles = (userId: string) => { setSelectedUserId(userId); setIsRoleModalOpen(true); };
  const handleRemoveMember = async (userId: string) => { if (!confirm(t("members.table.confirm_remove"))) return; try { await removeMember(userId); } catch (e: any) { console.error(e); } };

  if (isLoading) return <div className="flex justify-center py-32"><div className="animate-spin rounded-full h-12 w-12 border-4 border-primary-600 border-t-transparent shadow-xl"></div></div>;

  return (
    <>
      <div className="bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in duration-700">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("common.user")}</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("common.role")}</th>
                <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
              {members.map((member) => (
                <tr key={member.user_id} className="group hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all">
                  <td className="px-10 py-6">
                    <div className="flex items-center gap-5">
                      <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold shadow-inner border border-primary-100 dark:border-primary-800/50 overflow-hidden group-hover:scale-110 transition-transform">
                        {member.avatar_url ? (
                          <img src={member.avatar_url} alt="" className="w-full h-full object-cover" />
                        ) : (
                          <span className="text-lg uppercase">{(member.full_name || member.email).substring(0, 2)}</span>
                        )}
                      </div>
                      <div>
                        <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1.5">{member.full_name || t("common.no_name")}</div>
                        <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold flex items-center gap-2 opacity-70">
                          <Mail size={12} className="text-primary-400" /> {member.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6">
                    <span className={cn("inline-flex items-center gap-2 px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", 
                      member.role === "owner" ? "bg-purple-50 dark:bg-purple-950 text-purple-600 dark:text-purple-300 border-purple-100" : 
                      member.role === "admin" ? "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800" : 
                      "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800"
                    )}>
                      {member.role === "owner" && <Shield size={10} />}
                      {member.role.toUpperCase()}
                    </span>
                  </td>
                  <td className="px-10 py-6 text-right">
                    <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
                      <button onClick={() => handleManageRoles(member.user_id)} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("members.table.manage_roles")}><UserCog size={20} /></button>
                      {member.role !== "owner" && (
                        <button onClick={() => handleRemoveMember(member.user_id)} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("members.table.remove_member")}><Trash2 size={20} /></button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {members.length === 0 && (
            <div className="py-32 text-center bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
              <UserIcon className="w-20 h-20 mb-6 text-slate-200 dark:text-slate-700" />
              <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-[0.25em]">{t("members.table.messages.no_members")}</p>
            </div>
          )}
        </div>
      </div>

      {selectedUserId && (
        <RoleManagementModal isOpen={isRoleModalOpen} onClose={() => setIsRoleModalOpen(false)} userId={selectedUserId} />
      )}
    </>
  );
}

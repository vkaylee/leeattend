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
import RoleManagementModal from "@/components/RoleManagementModal";

export default function MemberTable() {
  const { t } = useTranslation();
  const { members, isLoading, removeMember } = useWorkspaceMembers();
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);

  const handleManageRoles = (userId: string) => { setSelectedUserId(userId); setIsRoleModalOpen(true); };
  const handleRemoveMember = async (userId: string) => { if (!confirm(t("members.table.confirm_remove"))) return; try { await removeMember(userId); } catch (e: any) { } };

  if (isLoading) return <div className="flex justify-center py-20"><div className="animate-spin rounded-full h-10 w-10 border-4 border-primary-600 border-t-transparent"></div></div>;

  return (
    <>
      <div className="bg-white rounded-[2rem] border border-slate-100 shadow-sm overflow-hidden animate-in fade-in duration-700">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-slate-100 bg-slate-50/50">
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.user")}</th>
                <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.role")}</th>
                <th className="px-8 py-5 border-b border-slate-100"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {members.map((member) => (
                <tr key={member.user_id} className="group hover:bg-slate-50/50 transition-all">
                  <td className="px-8 py-6">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 font-black shadow-inner border border-primary-100/50 overflow-hidden">
                        {member.avatar_url ? (
                          <img src={member.avatar_url} alt="" className="w-full h-full object-cover" />
                        ) : (
                          <span className="text-sm uppercase">{(member.full_name || member.email).substring(0, 2)}</span>
                        )}
                      </div>
                      <div>
                        <div className="font-black text-slate-900 text-sm tracking-tight uppercase">{member.full_name || t("common.no_name")}</div>
                        <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1.5 mt-1">
                          <Mail size={10} className="opacity-50" /> {member.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-6">
                    <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", 
                      member.role === "owner" ? "bg-purple-50 text-purple-600 border-purple-100" : 
                      member.role === "admin" ? "bg-blue-50 text-blue-600 border-blue-100" : 
                      "bg-slate-50 text-slate-500 border-slate-100"
                    )}>
                      {member.role === "owner" && <Shield size={10} />}
                      {member.role}
                    </span>
                  </td>
                  <td className="px-8 py-6 text-right">
                    <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300">
                      <button onClick={() => handleManageRoles(member.user_id)} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("members.table.manage_roles")}><UserCog size={18} /></button>
                      {member.role !== "owner" && (
                        <button onClick={() => handleRemoveMember(member.user_id)} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("members.table.remove_member")}><Trash2 size={18} /></button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {members.length === 0 && (
            <div className="py-24 text-center bg-slate-50/30">
              <UserIcon className="w-16 h-16 mx-auto mb-4 text-slate-200" />
              <p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("members.table.messages.no_members")}</p>
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

      {selectedUserId && (
        <RoleManagementModal
          isOpen={isRoleModalOpen}
          onClose={() => setIsRoleModalOpen(false)}
          userId={selectedUserId}
        />
      )}
    </>
  );
}

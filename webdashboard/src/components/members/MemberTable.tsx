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

  const handleManageRoles = (userId: string) => {
    setSelectedUserId(userId);
    setIsRoleModalOpen(true);
  };

  const handleRemoveMember = async (userId: string) => {
    if (!confirm(t("members.table.confirm_remove"))) return;
    try {
      await removeMember(userId);
    } catch (e: any) {
      alert(e.message || t("members.table.messages.remove_error"));
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <>
      <div className="bg-white rounded-2xl border border-slate-100 shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-slate-50 bg-slate-50/50">
                <th className="p-4 text-xs font-bold text-slate-500 uppercase tracking-widest">
                  {t("common.user")}
                </th>
                <th className="p-4 text-xs font-bold text-slate-500 uppercase tracking-widest">
                  {t("common.role")}
                </th>
                <th className="p-4 text-xs font-bold text-slate-500 uppercase tracking-widest text-right">
                  {t("common.actions")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {members.map((member) => (
                <tr
                  key={member.user_id}
                  className="group hover:bg-slate-50/50 transition-colors"
                >
                  <td className="p-4">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-full bg-indigo-100 flex items-center justify-center text-primary-600 font-bold">
                        {member.avatar_url ? (
                          <img
                            src={member.avatar_url}
                            alt=""
                            className="w-full h-full rounded-full object-cover"
                          />
                        ) : (
                          <span className="text-sm">
                            {(member.full_name || member.email)
                              .substring(0, 2)
                              .toUpperCase()}
                          </span>
                        )}
                      </div>
                      <div>
                        <div className="font-bold text-slate-700 text-sm">
                          {member.full_name || t("common.no_name")}
                        </div>
                        <div className="text-xs text-slate-400 flex items-center gap-1">
                          <Mail className="w-3 h-3" />
                          {member.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="p-4">
                    <div className="flex flex-wrap gap-2">
                      <span
                        className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-bold capitalize
                        ${
                          member.role === "owner"
                            ? "bg-purple-100 text-purple-700"
                            : member.role === "admin"
                              ? "bg-blue-100 text-blue-700"
                              : "bg-slate-100 text-slate-600"
                        }`}
                      >
                        {member.role === "owner" && (
                          <Shield className="w-3 h-3" />
                        )}
                        {member.role}
                      </span>
                    </div>
                  </td>
                  <td className="p-4 text-right">
                    <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={() => handleManageRoles(member.user_id)}
                        className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-colors"
                        title={t("members.table.manage_roles")}
                      >
                        <UserCog className="w-4 h-4" />
                      </button>
                      {member.role !== "owner" && (
                        <button
                          onClick={() => handleRemoveMember(member.user_id)}
                          className="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          title={t("members.table.remove_member")}
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {members.length === 0 && (
            <div className="p-12 text-center text-slate-400">
              <UserIcon className="w-12 h-12 mx-auto mb-4 text-slate-200" />
              <p>{t("members.table.messages.no_members")}</p>
            </div>
          )}
        </div>
      </div>

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

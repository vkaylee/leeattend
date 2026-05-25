import {

  Shield,
  Trash2,
  UserCog,
  KeyRound,
  Mail,
  User as UserIcon,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { TableSkeleton } from "@/components/common/table-skeleton";
import PermissionGate from "@/components/common/permission-gate";
import PermissionManagementModal from "@/components/features/settings/permission-management-modal";
import RoleManagementModal from "@/components/features/settings/role-management-modal";
import { useConfirm } from "@/hooks/use-confirm";
import { useWorkspaceMembers, useWorkspaceRoles } from "@/hooks/use-workspace";
import { WorkspaceRole, WorkspaceMember } from "@/lib/api";
import { cn } from "@/lib/utils";

const roleColorMap: Record<string, string> = {
  teal: "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100",
  blue: "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800",
  slate: "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
  rose: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100",
  primary: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 border-primary-100",
  emerald: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100",
};

function MemberRoleCell({ member, roles }: { member: WorkspaceMember; roles: WorkspaceRole[] }) {
  const primaryRole = roles.find((r) => r.id === member.role_id);
  if (!primaryRole) return <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-full animate-pulse"></div>;

  const extraRoles = (member.extra_role_ids ?? [])
    .map((id) => roles.find((r) => r.id === id))
    .filter(Boolean) as WorkspaceRole[];

  return (
    <>
      <span
        className={cn(
          "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-widest border shadow-sm",
          roleColorMap[primaryRole.color] || roleColorMap.slate
        )}
      >
        {primaryRole.is_system && primaryRole.name.toLowerCase() === "owner" ? <Shield size={10} /> : null}
        {primaryRole.name}
      </span>

      {extraRoles.length > 0 && (
        <div className="relative group/badge">
          <span className="inline-flex items-center justify-center min-w-[28px] h-[26px] px-1.5 rounded-full text-[10px] font-bold bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-slate-700 cursor-help transition-colors group-hover/badge:bg-primary-50 group-hover/badge:text-primary-600 group-hover/badge:border-primary-200 dark:group-hover/badge:bg-primary-900/50 dark:group-hover/badge:border-primary-800">
            +{extraRoles.length}
          </span>

          {/* Premium Popover for extra roles */}
          <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-max max-w-[200px] opacity-0 invisible group-hover/badge:opacity-100 group-hover/badge:visible transition-all duration-300 ease-out translate-y-2 group-hover/badge:translate-y-0 z-10">
            <div className="p-3 bg-white/90 dark:bg-slate-900/90 backdrop-blur-xl rounded-2xl shadow-xl shadow-slate-200/50 dark:shadow-black/40 border border-slate-100 dark:border-slate-800 flex flex-col gap-2">
              <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-slate-800">
                Extra Roles
              </span>
              {extraRoles.map((er) => (
                <span
                  className={cn(
                    "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border",
                    roleColorMap[er.color] || roleColorMap.slate
                  )}
                  key={er.id}
                >
                  {er.name}
                </span>
              ))}
            </div>
          </div>
        </div>
      )}
    </>
  );
}

export default function MemberTable() {
  const { t } = useTranslation();
  const { confirm, ConfirmationDialog } = useConfirm();
  const { members, isLoading: membersLoading, removeMember } = useWorkspaceMembers();
  const { roles, isLoading: rolesLoading } = useWorkspaceRoles();
  const isLoading = membersLoading || rolesLoading;
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);
  const [isPermModalOpen, setIsPermModalOpen] = useState(false);

  const handleManageRoles = (userId: string) => { setSelectedUserId(userId); setIsRoleModalOpen(true); };
  const handleManagePermissions = (userId: string) => { setSelectedUserId(userId); setIsPermModalOpen(true); };
  const handleRemoveMember = async (userId: string) => { 
    const isConfirmed = await confirm({ title: t('common.warning', { defaultValue: 'Warning' }), message: t("members.table.confirm_remove"), variant: 'danger' });
    if (!isConfirmed) return; 
 
    try { await removeMember(userId); } catch (e: unknown) { console.error(e); } 
  };

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
              {isLoading ? (
                <TableSkeleton rows={5} />
              ) : members.length === 0 ? (
                <tr>
                  <td colSpan={3}>
                    <div className="py-32 text-center bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
                      <UserIcon className="w-20 h-20 mb-6 text-slate-200 dark:text-slate-700" />
                      <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-[0.25em]">{t("members.table.messages.no_members")}</p>
                    </div>
                  </td>
                </tr>
              ) : (
                members.map((member) => (
                  <tr className="group hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all" key={member.user_id}>
                    <td className="px-10 py-6">
                      <div className="flex items-center gap-5">
                        <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold shadow-inner border border-primary-100 dark:border-primary-800/50 overflow-hidden group-hover:scale-110 transition-transform">
                          {member.avatar_url ? (
                            <img alt="Member avatar" className="w-full h-full object-cover" src={member.avatar_url} />
                          ) : (
                            <span className="text-lg uppercase">{(member.full_name ?? member.email).substring(0, 2)}</span>
                          )}
                        </div>
                        <div>
                          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1.5">{member.full_name ?? t("common.no_name")}</div>
                          <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold flex items-center gap-2 opacity-70">
                            <Mail className="text-primary-400" size={12} /> {member.email}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-6">
                      <div className="flex flex-wrap items-center gap-2">
                        <MemberRoleCell member={member} roles={roles} />
                      </div>
                    </td>
                    <td className="px-10 py-6 text-right">
                      <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
                        <PermissionGate permission="members:roles:manage">
                          <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { handleManageRoles(member.user_id); }} title={t("members.table.manage_roles")}><UserCog size={20} /></button>
                        </PermissionGate>
                        <PermissionGate permission="members:roles:manage">
                          <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-amber-600 dark:text-amber-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { handleManagePermissions(member.user_id); }} title={t("members.table.manage_permissions", { defaultValue: "Manage Permissions" })}><KeyRound size={20} /></button>
                        </PermissionGate>
                        {roles.find(r => r.id === member.role_id)?.name.toLowerCase() !== "owner" && (
                          <PermissionGate permission="members:remove">
                            <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => handleRemoveMember(member.user_id)} title={t("members.table.remove_member")}><Trash2 size={20} /></button>
                          </PermissionGate>
                        )}
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {!!selectedUserId && <>
          <RoleManagementModal isOpen={isRoleModalOpen} onClose={() => { setIsRoleModalOpen(false); }} userId={selectedUserId} />
          <PermissionManagementModal isOpen={isPermModalOpen} onClose={() => { setIsPermModalOpen(false); }} userId={selectedUserId} />
        </>}
      <ConfirmationDialog />
    </>
  );
}

import { Plus, Shield, ShieldAlert, ShieldCheck, ShieldHalf, Trash2, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface Assignee {
  assigned_at: string;
  user_id: string;
}

interface Role {
  assignees: Assignee[];
  description?: string;
  id: string;
  name: string;
}

interface SystemRoleCardProps {
  isRevoking: boolean;
  onAssign: (roleName: string) => void;
  onRevoke: (userId: string, roleName: string) => void;
  role: Role;
}

const roleDescriptions: Record<string, string> = {
  billing_admin: "system_roles.descriptions.billing_admin",
  platform_admin: "system_roles.descriptions.platform_admin",
  support: "system_roles.descriptions.support",
};

const roleIcons: Record<string, React.ElementType> = {
  billing_admin: ShieldHalf,
  platform_admin: ShieldAlert,
  support: ShieldCheck,
};

export function SystemRoleCard({ isRevoking, onAssign, onRevoke, role }: SystemRoleCardProps) {
  const { t } = useTranslation();
  const Icon = (roleIcons[role.name] as React.ElementType | undefined) ?? Shield;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2rem] border border-slate-100 dark:border-slate-800 p-5 sm:p-6 shadow-sm hover:shadow-xl hover:shadow-slate-200/50 dark:hover:shadow-black/20 transition-all duration-300 group flex flex-col">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-4">
          <div className="w-14 h-14 rounded-2xl flex items-center justify-center border bg-blue-50 text-blue-600 dark:bg-blue-500/10 dark:text-blue-400 border-blue-200/50 dark:border-blue-500/20">
            <Icon size={24} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
                {role.name}
              </Typography>
              <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-rose-100 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400">
                {t("system_roles.system_label")}
              </span>
            </div>
            <div className="flex items-center gap-1.5 text-sm font-medium text-slate-500 mt-1">
              <Users size={14} />
              <span>{t("system_roles.assignees_count", { count: role.assignees.length })}</span>
            </div>
          </div>
        </div>
        
        <Button className="p-2 text-blue-600 bg-blue-50 hover:bg-blue-100 dark:bg-blue-500/10 dark:text-blue-400 dark:hover:bg-blue-500/20 rounded-xl transition-colors"
          onClick={() => { onAssign(role.name); }}
          title={t("system_roles.assign_to_user")}
          variant="ghost"
        >
          <Plus size={20} />
        </Button>
      </div>

      <Typography className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-6 flex-1" variant="p">
        {role.description ?? t(roleDescriptions[role.name] ?? "system_roles.descriptions.default")}
      </Typography>

      <div className="space-y-3 pt-4 border-t border-slate-100 dark:border-slate-800">
        <Typography className="text-xs font-bold text-slate-400 uppercase tracking-wider" variant="h4">{t("system_roles.assigned_users")}</Typography>
        {role.assignees.length === 0 ? (
          <Typography className="text-sm text-slate-500 italic" variant="p">{t("system_roles.no_users")}</Typography>
        ) : (
          <div className="space-y-2 max-h-[200px] overflow-y-auto pr-2 custom-scrollbar">
            {role.assignees.map((assignee) => (
              <div className="flex items-center justify-between p-3 rounded-xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800" key={assignee.user_id}>
                <div className="flex flex-col">
                  <span className="text-sm font-bold text-slate-700 dark:text-slate-300 font-mono">
                    {assignee.user_id}
                  </span>
                  <span className="text-xs text-slate-500">
                    {t("system_roles.assigned_date", { date: new Date(assignee.assigned_at).toLocaleDateString() })}
                  </span>
                </div>
                <Button className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-500/10 rounded-lg transition-colors disabled:opacity-50"
                  disabled={isRevoking}
                  onClick={() => { onRevoke(assignee.user_id, role.name); }}
                  title={t("system_roles.revoke_role")}
                  variant="ghost"
                >
                  <Trash2 size={16} />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

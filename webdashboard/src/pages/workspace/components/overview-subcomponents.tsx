import { Building2, CalendarDays, Clock, Users, BarChart3 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

import type { Workspace, WorkspaceMember, WorkspaceRole } from "../../../lib/api/types";

interface WorkspaceHeroCardProps {
  workspace: Workspace;
}

export const WorkspaceHeroCard = ({ workspace }: WorkspaceHeroCardProps) => {
  const { t } = useTranslation();
  
  return (
    <div className="relative overflow-hidden rounded-[2.5rem] bg-slate-900 p-10 text-white shadow-xl shadow-slate-200 dark:shadow-none">
      <div className="absolute top-0 right-0 p-8 opacity-10">
        <Building2 size={200} />
      </div>
      <div className="relative z-10">
        <div className="inline-flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/10 backdrop-blur-md border border-white/20 mb-6">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span className="text-xs font-bold uppercase tracking-widest text-slate-100">
            {t('overview.activeWorkspace', { defaultValue: 'Active Workspace' })}
          </span>
        </div>
        <Typography className="text-5xl font-black tracking-tight mb-2" variant="h2">{workspace.name}</Typography>
        <div className="flex items-center gap-6 mt-8 text-slate-300 font-medium">
          <div className="flex items-center gap-2">
            <Clock className="opacity-70" size={16} />
            <span>{t('overview.timezone', { defaultValue: 'Timezone:' })} {workspace.timezone}</span>
          </div>
          <div className="flex items-center gap-2">
            <CalendarDays className="opacity-70" size={16} />
            <span>{t('overview.created', { defaultValue: 'Created:' })} {new Date(workspace.created_at).toLocaleDateString()}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

interface WorkspaceStats {
  late_today: number;
  on_leave_today: number;
  present_today: number;
  total_employees: number;
}

interface WorkspaceStatsGridProps {
  stats: WorkspaceStats | undefined;
}

export const WorkspaceStatsGrid = ({ stats }: WorkspaceStatsGridProps) => {
  const { t } = useTranslation();
  
  const statItems = [
    {
      icon: Users,
      value: stats?.total_employees ?? 0,
      label: t('overview.totalEmployees', { defaultValue: 'Total Employees' }),
      colorClass: "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-400",
    },
    {
      icon: BarChart3,
      value: stats?.present_today ?? 0,
      label: t('overview.presentToday', { defaultValue: 'Present Today' }),
      colorClass: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400",
    },
    {
      icon: Clock,
      value: stats?.late_today ?? 0,
      label: t('overview.lateToday', { defaultValue: 'Late Today' }),
      colorClass: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-400",
    },
    {
      icon: CalendarDays,
      value: stats?.on_leave_today ?? 0,
      label: t('overview.onLeave', { defaultValue: 'On Leave' }),
      colorClass: "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-400",
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      {statItems.map((item, idx) => (
        <div className="p-6 bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm" key={idx}>
          <div className={`w-12 h-12 rounded-2xl flex items-center justify-center mb-4 ${item.colorClass}`}>
            <item.icon size={24} />
          </div>
          <div className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight">
            {item.value}
          </div>
          <div className="text-xs font-bold text-slate-500 uppercase tracking-widest mt-1">
            {item.label}
          </div>
        </div>
      ))}
    </div>
  );
};

interface AdminMembersPreviewProps {
  members: WorkspaceMember[];
  roles: WorkspaceRole[];
}

export const AdminMembersPreview = ({ members, roles }: AdminMembersPreviewProps) => {
  const { t } = useTranslation();
  
  const admins = members.filter(m => {
    const r = roles.find(role => role.id === m.role_id);
    return r && (r.name.toLowerCase() === 'owner' || r.name.toLowerCase() === 'admin');
  });

  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2.5rem] p-8 border border-slate-100 dark:border-slate-800">
      <div className="flex items-center justify-between mb-6">
        <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
          {t('overview.adminMembers', { defaultValue: 'Admin Members' })}
        </Typography>
        <span className="px-3 py-1 bg-white dark:bg-slate-900 rounded-xl text-[10px] font-bold text-slate-500 uppercase tracking-widest shadow-sm">
          {admins.length} {t('overview.admins', { defaultValue: 'Admins' })}
        </span>
      </div>
      <div className="flex flex-wrap gap-4">
        {admins.map(admin => (
          <div className="flex items-center gap-3 bg-white dark:bg-slate-900 px-4 py-3 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-800" key={admin.user_id}>
            <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold">
              {admin.avatar_url ? (
                <img alt="Admin avatar" className="w-full h-full object-cover rounded-xl" src={admin.avatar_url} />
              ) : (
                (admin.full_name ?? admin.email).substring(0, 2).toUpperCase()
              )}
            </div>
            <div>
              <div className="text-sm font-bold text-slate-700 dark:text-slate-200">
                {admin.full_name ?? t('overview.adminFallback', { defaultValue: 'Admin' })}
              </div>
              <div className="text-[10px] font-medium text-slate-500">{admin.email}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

import { useWorkspace, useWorkspaceMembers, useWorkspaceRoles } from "@/hooks/use-workspace";
import PageHeader from "@/components/ui/PageHeader";
import { Building2, Users, CalendarDays, BarChart3, Clock, Loader2 } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { apiFetch, API_BASE_URL, getCurrentWorkspaceId } from "@/lib/api";

export default function WorkspaceOverviewPage() {
  const wsId = getCurrentWorkspaceId() || "";
  const { workspace, isLoading: isWsLoading } = useWorkspace();
  const { members, isLoading: isMembersLoading } = useWorkspaceMembers();
  const { roles, isLoading: isRolesLoading } = useWorkspaceRoles();

  // Fetch workspace stats
  const { data: stats, isLoading: isStatsLoading } = useQuery({
    queryKey: ["workspace", wsId, "stats"],
    queryFn: async () => {
      const res = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/stats`);
      if (!res.ok) throw new Error("Failed to fetch stats");
      return res.json();
    },
    enabled: !!wsId,
  });

  if (isWsLoading || isMembersLoading || isStatsLoading || isRolesLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  if (!workspace) return null;

  return (
    <div className="space-y-10 animate-in fade-in duration-500">
      <PageHeader
        title="Workspace Overview"
        subtitle={`ID: ${workspace.id.substring(0, 8).toUpperCase()}`}
      />

      {/* Hero Card */}
      <div className="relative overflow-hidden rounded-[2.5rem] bg-indigo-600 p-10 text-white shadow-xl shadow-indigo-200 dark:shadow-none">
        <div className="absolute top-0 right-0 p-8 opacity-10">
          <Building2 size={200} />
        </div>
        <div className="relative z-10">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/10 backdrop-blur-md border border-white/20 mb-6">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span className="text-xs font-bold uppercase tracking-widest text-indigo-50">Active Workspace</span>
          </div>
          <h2 className="text-5xl font-black tracking-tight mb-2">{workspace.name}</h2>
          <div className="flex items-center gap-6 mt-8 text-indigo-100 font-medium">
            <div className="flex items-center gap-2">
              <Clock size={16} className="opacity-70" />
              <span>Timezone: {workspace.timezone}</span>
            </div>
            <div className="flex items-center gap-2">
              <CalendarDays size={16} className="opacity-70" />
              <span>Created: {new Date(workspace.created_at).toLocaleDateString()}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="p-6 bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm">
          <div className="w-12 h-12 bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-400 rounded-2xl flex items-center justify-center mb-4">
            <Users size={24} />
          </div>
          <div className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight">
            {stats?.total_employees || 0}
          </div>
          <div className="text-xs font-bold text-slate-500 uppercase tracking-widest mt-1">
            Total Employees
          </div>
        </div>

        <div className="p-6 bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm">
          <div className="w-12 h-12 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 rounded-2xl flex items-center justify-center mb-4">
            <BarChart3 size={24} />
          </div>
          <div className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight">
            {stats?.present_today || 0}
          </div>
          <div className="text-xs font-bold text-slate-500 uppercase tracking-widest mt-1">
            Present Today
          </div>
        </div>

        <div className="p-6 bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm">
          <div className="w-12 h-12 bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-400 rounded-2xl flex items-center justify-center mb-4">
            <Clock size={24} />
          </div>
          <div className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight">
            {stats?.late_today || 0}
          </div>
          <div className="text-xs font-bold text-slate-500 uppercase tracking-widest mt-1">
            Late Today
          </div>
        </div>

        <div className="p-6 bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm">
          <div className="w-12 h-12 bg-purple-50 dark:bg-purple-950 text-purple-600 dark:text-purple-400 rounded-2xl flex items-center justify-center mb-4">
            <CalendarDays size={24} />
          </div>
          <div className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight">
            {stats?.on_leave_today || 0}
          </div>
          <div className="text-xs font-bold text-slate-500 uppercase tracking-widest mt-1">
            On Leave
          </div>
        </div>
      </div>

      {/* Admin Members Preview */}
      <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2.5rem] p-8 border border-slate-100 dark:border-slate-800">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200">Admin Members</h3>
          <span className="px-3 py-1 bg-white dark:bg-slate-900 rounded-xl text-[10px] font-bold text-slate-500 uppercase tracking-widest shadow-sm">
            {members.filter(m => {
              const r = roles.find(role => role.id === m.role_id);
              return r && (r.name.toLowerCase() === 'owner' || r.name.toLowerCase() === 'admin');
            }).length} Admins
          </span>
        </div>
        <div className="flex flex-wrap gap-4">
          {members.filter(m => {
            const r = roles.find(role => role.id === m.role_id);
            return r && (r.name.toLowerCase() === 'owner' || r.name.toLowerCase() === 'admin');
          }).map(admin => (
            <div key={admin.user_id} className="flex items-center gap-3 bg-white dark:bg-slate-900 px-4 py-3 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-800">
              <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold">
                {admin.avatar_url ? (
                  <img src={admin.avatar_url} alt="" className="w-full h-full object-cover rounded-xl" />
                ) : (
                  (admin.full_name || admin.email).substring(0, 2).toUpperCase()
                )}
              </div>
              <div>
                <div className="text-sm font-bold text-slate-700 dark:text-slate-200">{admin.full_name || "Admin"}</div>
                <div className="text-[10px] font-medium text-slate-500">{admin.email}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

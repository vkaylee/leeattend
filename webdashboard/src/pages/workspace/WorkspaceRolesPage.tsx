import React, { useState } from "react";
import PageHeader from "@/components/ui/PageHeader";
import { 
  ShieldAlert, 
  Shield, 
  ShieldCheck, 
  User, 
  Plus, 
  MoreVertical, 
  Edit2, 
  Copy, 
  Trash2,
  Users,
  CheckCircle2,
  ChevronRight
} from "lucide-react";
import { cn } from "@/lib/utils";

// Mock Data
const mockRoles = [
  {
    id: "r1",
    name: "Workspace Admin",
    description: "Full access to all workspace settings, billing, and danger zone operations.",
    icon: ShieldAlert,
    color: "rose",
    userCount: 2,
    isSystem: true,
    keyPermissions: ["All Access", "Billing", "System Config"],
    users: [
      { id: "u1", name: "Alice", avatar: "A" },
      { id: "u2", name: "Bob", avatar: "B" }
    ]
  },
  {
    id: "r2",
    name: "HR Manager",
    description: "Can manage people, time tracking, leave policies, and organizational structure.",
    icon: ShieldCheck,
    color: "indigo",
    userCount: 5,
    isSystem: false,
    keyPermissions: ["Manage Employees", "Approve Leaves", "Edit Policies"],
    users: [
      { id: "u3", name: "Charlie", avatar: "C" },
      { id: "u4", name: "Diana", avatar: "D" },
      { id: "u5", name: "Eve", avatar: "E" }
    ]
  },
  {
    id: "r3",
    name: "Payroll Specialist",
    description: "Read-only access to time tracking, full access to payroll processing.",
    icon: Shield,
    color: "emerald",
    userCount: 1,
    isSystem: false,
    keyPermissions: ["Run Payroll", "View Attendance", "View Reports"],
    users: [
      { id: "u6", name: "Frank", avatar: "F" }
    ]
  },
  {
    id: "r4",
    name: "Employee",
    description: "Default role. Can access Employee Self-Service (ESS), clock in/out, and request leaves.",
    icon: User,
    color: "slate",
    userCount: 142,
    isSystem: true,
    keyPermissions: ["ESS Access", "Submit Leave", "Clock In/Out"],
    users: [
      { id: "u7", name: "Grace", avatar: "G" },
      { id: "u8", name: "Hank", avatar: "H" },
      { id: "u9", name: "Ivy", avatar: "I" }
    ]
  }
];

const colorMap: Record<string, string> = {
  rose: "bg-rose-50 text-rose-600 dark:bg-rose-500/10 dark:text-rose-400 border-rose-200/50 dark:border-rose-500/20",
  indigo: "bg-indigo-50 text-indigo-600 dark:bg-indigo-500/10 dark:text-indigo-400 border-indigo-200/50 dark:border-indigo-500/20",
  emerald: "bg-emerald-50 text-emerald-600 dark:bg-emerald-500/10 dark:text-emerald-400 border-emerald-200/50 dark:border-emerald-500/20",
  slate: "bg-slate-50 text-slate-600 dark:bg-slate-500/10 dark:text-slate-400 border-slate-200/50 dark:border-slate-500/20"
};

const iconColorMap: Record<string, string> = {
  rose: "text-rose-500 dark:text-rose-400",
  indigo: "text-indigo-500 dark:text-indigo-400",
  emerald: "text-emerald-500 dark:text-emerald-400",
  slate: "text-slate-500 dark:text-slate-400"
};

export default function WorkspaceRolesPage() {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          title="Role Management"
          subtitle="Define custom roles and manage access levels across the workspace"
        />
        <button className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-5 py-2.5 rounded-2xl font-bold text-sm transition-all shadow-lg shadow-indigo-200 dark:shadow-indigo-900/20 hover:shadow-xl hover:-translate-y-0.5 active:translate-y-0">
          <Plus size={18} />
          Create Role
        </button>
      </div>
      
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {mockRoles.map((role) => {
          const Icon = role.icon;
          const bgColors = colorMap[role.color] || colorMap.slate;
          const iconColors = iconColorMap[role.color] || iconColorMap.slate;
          
          return (
            <div 
              key={role.id}
              className="bg-white dark:bg-slate-900 rounded-[2rem] border border-slate-100 dark:border-slate-800 p-6 shadow-sm hover:shadow-xl hover:shadow-slate-200/50 dark:hover:shadow-black/20 transition-all duration-300 group"
            >
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-4">
                  <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center border", bgColors)}>
                    <Icon size={24} className={iconColors} />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200">
                        {role.name}
                      </h3>
                      {role.isSystem && (
                        <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400">
                          System
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1.5 text-sm font-medium text-slate-500 mt-1">
                      <Users size={14} />
                      <span>{role.userCount} users</span>
                    </div>
                  </div>
                </div>
                
                {/* Action Menu */}
                <div className="relative">
                  <button 
                    onClick={() => setActiveMenu(activeMenu === role.id ? null : role.id)}
                    className="p-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-colors"
                  >
                    <MoreVertical size={20} />
                  </button>
                  
                  {activeMenu === role.id && (
                    <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-slate-900 rounded-2xl shadow-xl shadow-slate-200/50 dark:shadow-black/40 border border-slate-100 dark:border-slate-800 py-2 z-10 animate-in fade-in zoom-in-95 duration-200">
                      <button className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                        <Edit2 size={16} /> Edit Role
                      </button>
                      <button className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                        <Copy size={16} /> Duplicate
                      </button>
                      {!role.isSystem && (
                        <>
                          <div className="h-px bg-slate-100 dark:bg-slate-800 my-1"></div>
                          <button className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-500/10 transition-colors">
                            <Trash2 size={16} /> Delete Role
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </div>
              </div>

              <p className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-6 line-clamp-2 min-h-[40px]">
                {role.description}
              </p>

              <div className="space-y-4">
                <div>
                  <h4 className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-3">Key Permissions</h4>
                  <div className="flex flex-wrap gap-2">
                    {role.keyPermissions.map((perm, idx) => (
                      <div key={idx} className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800">
                        <CheckCircle2 size={12} className="text-emerald-500" />
                        <span className="text-xs font-bold text-slate-600 dark:text-slate-300">{perm}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="pt-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between">
                  <div className="flex -space-x-3">
                    {role.users.map((user, idx) => (
                      <div 
                        key={idx} 
                        className="w-8 h-8 rounded-full border-2 border-white dark:border-slate-900 bg-gradient-to-tr from-slate-200 to-slate-300 dark:from-slate-700 dark:to-slate-600 flex items-center justify-center text-[10px] font-bold text-slate-600 dark:text-slate-200 shadow-sm"
                        title={user.name}
                      >
                        {user.avatar}
                      </div>
                    ))}
                    {role.userCount > 3 && (
                      <div className="w-8 h-8 rounded-full border-2 border-white dark:border-slate-900 bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-[10px] font-bold text-slate-500 shadow-sm">
                        +{role.userCount - 3}
                      </div>
                    )}
                  </div>
                  
                  <button className="text-xs font-bold text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1">
                    Manage Access <ChevronRight size={14} />
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}


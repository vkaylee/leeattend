import React, { useState } from "react";
import PageHeader from "@/components/ui/PageHeader";
import { 
  KeyRound,
  Save,
  Check,
  Building2,
  Users,
  Clock,
  Banknote,
  ShieldCheck,
  Info
} from "lucide-react";
import { cn } from "@/lib/utils";

// Mock Data
const roles = [
  { id: "admin", name: "Admin", isSystem: true, color: "rose" },
  { id: "hr", name: "HR Manager", isSystem: false, color: "indigo" },
  { id: "payroll", name: "Payroll", isSystem: false, color: "emerald" },
  { id: "employee", name: "Employee", isSystem: true, color: "slate" },
];

const permissionGroups = [
  {
    id: "general",
    name: "General & Setup",
    icon: Building2,
    permissions: [
      { id: "edit_workspace", name: "Edit Workspace Info", desc: "Change name, timezone, logo" },
      { id: "manage_billing", name: "Manage Billing", desc: "Update cards, view invoices" },
      { id: "manage_roles", name: "Manage Roles", desc: "Create or delete custom roles" },
    ]
  },
  {
    id: "people",
    name: "People Management",
    icon: Users,
    permissions: [
      { id: "view_employees", name: "View Employees", desc: "Access employee directory" },
      { id: "manage_employees", name: "Manage Employees", desc: "Add, edit, or terminate" },
      { id: "manage_org", name: "Manage Departments", desc: "Modify organizational structure" },
    ]
  },
  {
    id: "time",
    name: "Time & Attendance",
    icon: Clock,
    permissions: [
      { id: "view_time", name: "View Attendance", desc: "See everyone's time logs" },
      { id: "edit_time", name: "Edit Time Logs", desc: "Fix missing clock-ins" },
      { id: "approve_ot", name: "Approve Overtime", desc: "Review and approve OT requests" },
    ]
  },
  {
    id: "payroll",
    name: "Payroll",
    icon: Banknote,
    permissions: [
      { id: "run_payroll", name: "Run Payroll", desc: "Execute payroll processing" },
      { id: "view_salaries", name: "View Salaries", desc: "See compensation data" },
    ]
  },
  {
    id: "security",
    name: "Security",
    icon: ShieldCheck,
    permissions: [
      { id: "view_audit", name: "View Audit Logs", desc: "Access security event logs" },
      { id: "danger_zone", name: "Danger Zone", desc: "Data deletion and resets" },
    ]
  }
];

// Mock assigned permissions
const initialPermissions: Record<string, string[]> = {
  admin: [
    "edit_workspace", "manage_billing", "manage_roles", 
    "view_employees", "manage_employees", "manage_org",
    "view_time", "edit_time", "approve_ot",
    "run_payroll", "view_salaries",
    "view_audit", "danger_zone"
  ],
  hr: [
    "view_employees", "manage_employees", "manage_org",
    "view_time", "edit_time"
  ],
  payroll: [
    "view_employees", "view_time", "run_payroll", "view_salaries"
  ],
  employee: [
    "view_employees"
  ]
};

export default function WorkspacePermissionsPage() {
  const [permissions, setPermissions] = useState(initialPermissions);
  const [hasChanges, setHasChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const togglePermission = (roleId: string, permId: string) => {
    if (roleId === "admin") return; // Admin permissions cannot be changed

    setPermissions(prev => {
      const rolePerms = prev[roleId] || [];
      const newPerms = rolePerms.includes(permId)
        ? rolePerms.filter(p => p !== permId)
        : [...rolePerms, permId];
      
      setHasChanges(true);
      return { ...prev, [roleId]: newPerms };
    });
  };

  const handleSave = () => {
    setIsSaving(true);
    setTimeout(() => {
      setIsSaving(false);
      setHasChanges(false);
    }, 800);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-10">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          title="Permission Matrix"
          subtitle="Fine-tune access control for each role across all modules"
        />
        <div className="flex items-center gap-3">
          {hasChanges && (
            <span className="text-xs font-bold text-amber-500 animate-pulse">Unsaved changes</span>
          )}
          <button 
            onClick={handleSave}
            disabled={!hasChanges || isSaving}
            className={cn(
              "flex items-center gap-2 px-5 py-2.5 rounded-2xl font-bold text-sm transition-all",
              hasChanges 
                ? "bg-indigo-600 hover:bg-indigo-700 text-white shadow-lg shadow-indigo-200 dark:shadow-indigo-900/20 hover:-translate-y-0.5" 
                : "bg-slate-100 dark:bg-slate-800 text-slate-400 cursor-not-allowed"
            )}
          >
            {isSaving ? (
              <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              <Save size={18} />
            )}
            Save Matrix
          </button>
        </div>
      </div>
      
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden">
        <div className="overflow-x-auto custom-scrollbar">
          <table className="w-full text-left border-collapse min-w-[800px]">
            <thead>
              <tr>
                <th className="sticky left-0 z-20 bg-slate-50 dark:bg-slate-900 border-b border-r border-slate-100 dark:border-slate-800 p-6 min-w-[280px]">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-indigo-50 dark:bg-indigo-900/30 rounded-xl">
                      <KeyRound className="w-5 h-5 text-indigo-500" />
                    </div>
                    <div>
                      <h3 className="font-bold text-slate-800 dark:text-slate-200">Modules</h3>
                      <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-0.5">Permissions</p>
                    </div>
                  </div>
                </th>
                {roles.map(role => (
                  <th key={role.id} className="bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 p-6 text-center min-w-[140px]">
                    <div className="flex flex-col items-center justify-center gap-2">
                      <h4 className="font-bold text-sm text-slate-800 dark:text-slate-200">{role.name}</h4>
                      {role.isSystem ? (
                        <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500">System</span>
                      ) : (
                        <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400">Custom</span>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {permissionGroups.map((group, groupIdx) => (
                <React.Fragment key={group.id}>
                  {/* Group Header */}
                  <tr>
                    <td 
                      colSpan={roles.length + 1} 
                      className="bg-slate-50/50 dark:bg-slate-800/20 border-b border-slate-100 dark:border-slate-800 py-3 px-6"
                    >
                      <div className="flex items-center gap-2">
                        <group.icon className="w-4 h-4 text-slate-400" />
                        <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{group.name}</span>
                      </div>
                    </td>
                  </tr>
                  
                  {/* Permissions Rows */}
                  {group.permissions.map((perm, permIdx) => (
                    <tr key={perm.id} className="group hover:bg-slate-50 dark:hover:bg-slate-800/30 transition-colors">
                      <td className="sticky left-0 z-10 bg-white dark:bg-slate-900 group-hover:bg-slate-50 dark:group-hover:bg-slate-800 border-b border-r border-slate-100 dark:border-slate-800 p-4 pl-6">
                        <div className="flex items-start gap-2">
                          <div>
                            <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{perm.name}</p>
                            <p className="text-xs text-slate-500 mt-0.5">{perm.desc}</p>
                          </div>
                        </div>
                      </td>
                      
                      {roles.map(role => {
                        const isChecked = permissions[role.id]?.includes(perm.id);
                        const isAdmin = role.id === "admin";
                        
                        return (
                          <td key={`${role.id}-${perm.id}`} className="border-b border-slate-100 dark:border-slate-800 p-4 text-center">
                            <label className={cn(
                              "relative inline-flex items-center justify-center w-6 h-6 rounded-lg border-2 transition-all cursor-pointer",
                              isAdmin ? "cursor-not-allowed opacity-60" : "hover:border-indigo-400",
                              isChecked 
                                ? "bg-indigo-500 border-indigo-500 text-white" 
                                : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                            )}>
                              <input 
                                type="checkbox" 
                                className="sr-only"
                                checked={isChecked}
                                disabled={isAdmin}
                                onChange={() => togglePermission(role.id, perm.id)}
                              />
                              <Check className="w-4 h-4" strokeWidth={3} />
                            </label>
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </React.Fragment>
              ))}
            </tbody>
          </table>
        </div>
        
        <div className="bg-slate-50 dark:bg-slate-800/50 p-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-center gap-2">
          <Info className="w-4 h-4 text-slate-400" />
          <p className="text-xs font-medium text-slate-500">System roles (like Admin) have fixed permissions that cannot be modified.</p>
        </div>
      </div>
    </div>
  );
}

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
  ChevronRight,
  PlusCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useWorkspaceRoles } from "@/hooks/use-workspace";
import RoleFormModal from "@/components/features/settings/RoleFormModal";
import { Link } from "@tanstack/react-router";
import PermissionGate from "@/components/common/PermissionGate";
import { useTranslation } from "react-i18next";
import { useConfirm } from "@/hooks/use-confirm";

// No mock roles needed

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
  const { t } = useTranslation();
  const { confirm, ConfirmationDialog } = useConfirm();
  const { roles, isLoading, deleteRole } = useWorkspaceRoles();
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<{ id: string; name: string; description: string; color: string } | null>(null);

  const handleCreate = () => {
    setEditingRole(null);
    setIsModalOpen(true);
  };

  const handleEdit = (role: any) => {
    setEditingRole(role);
    setIsModalOpen(true);
    setActiveMenu(null);
  };

  const handleDelete = async (roleId: string) => {
    const isConfirmed = await confirm({
      title: t('rolesPage.deleteTitle', { defaultValue: 'Delete Role' }),
      message: t('rolesPage.confirmDelete', { defaultValue: "Are you sure you want to delete this role?" }),
      variant: 'danger',
    });
    if (isConfirmed) {
      await deleteRole(roleId);
    }
    setActiveMenu(null);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <PageHeader
          title={t('rolesPage.title', { defaultValue: 'Role Management' })}
          subtitle={t('rolesPage.subtitle', { defaultValue: 'Define custom roles and manage access levels across the workspace' })}
        />
        <PermissionGate permission="members:roles:manage">
          <button onClick={handleCreate} className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-5 py-2.5 rounded-2xl font-bold text-sm transition-all shadow-lg shadow-indigo-200 dark:shadow-indigo-900/20 hover:shadow-xl hover:-translate-y-0.5 active:translate-y-0">
            <Plus size={18} />
            {t('rolesPage.createRole', { defaultValue: 'Create Role' })}
          </button>
        </PermissionGate>
      </div>
      
      {isLoading ? (
        <div className="flex justify-center p-20">
          <div className="w-10 h-10 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin"></div>
        </div>
      ) : (
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {roles.map((role) => {
          const Icon = role.is_system && role.name.toLowerCase() === 'admin' ? ShieldAlert 
                    : role.is_system && role.name.toLowerCase() === 'owner' ? ShieldCheck
                    : role.is_system ? User : Shield;
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
                      {role.is_system && (
                        <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400">
                          {t('common.system', { defaultValue: 'System' })}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1.5 text-sm font-medium text-slate-500 mt-1">
                      <Users size={14} />
                      <span>{t('rolesPage.roleId', { defaultValue: 'Role ID:' })} {role.id.split('-')[0]}...</span>
                    </div>
                  </div>
                </div>
                
                {/* Action Menu */}
                <PermissionGate permission="members:roles:manage">
                  <div className="relative">
                    <button 
                      onClick={() => setActiveMenu(activeMenu === role.id ? null : role.id)}
                      className="p-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-colors"
                    >
                      <MoreVertical size={20} />
                    </button>
                    
                    {activeMenu === role.id && (
                      <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-slate-900 rounded-2xl shadow-xl shadow-slate-200/50 dark:shadow-black/40 border border-slate-100 dark:border-slate-800 py-2 z-10 animate-in fade-in zoom-in-95 duration-200">
                        {!role.is_system && (
                          <button onClick={() => handleEdit(role)} className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                            <Edit2 size={16} /> {t('rolesPage.editRole', { defaultValue: 'Edit Role' })}
                          </button>
                        )}
                        <Link to="/workspace/settings/permissions" className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                          <CheckCircle2 size={16} /> {t('rolesPage.matrix', { defaultValue: 'Matrix' })}
                        </Link>
                        {!role.is_system && (
                          <>
                            <div className="h-px bg-slate-100 dark:bg-slate-800 my-1"></div>
                            <button onClick={() => handleDelete(role.id)} className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-500/10 transition-colors">
                              <Trash2 size={16} /> {t('rolesPage.deleteRole', { defaultValue: 'Delete Role' })}
                            </button>
                          </>
                        )}
                      </div>
                    )}
                  </div>
                </PermissionGate>
              </div>

              <p className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-6 line-clamp-2 min-h-[40px]">
                {role.description || t('rolesPage.noDescription', { defaultValue: 'No description provided.' })}
              </p>

              <div className="space-y-4">
                <div className="pt-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between">
                  <div className="flex -space-x-3">
                    <div className="w-8 h-8 rounded-full border-2 border-white dark:border-slate-900 bg-gradient-to-tr from-slate-200 to-slate-300 dark:from-slate-700 dark:to-slate-600 flex items-center justify-center text-[10px] font-bold text-slate-600 dark:text-slate-200 shadow-sm">
                      <User size={14} />
                    </div>
                  </div>
                  <PermissionGate permission="members:roles:manage">
                    <Link to="/workspace/settings/permissions" className="text-xs font-bold text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1">
                      {t('rolesPage.manageAccess', { defaultValue: 'Manage Access' })} <ChevronRight size={14} />
                    </Link>
                  </PermissionGate>
                </div>
              </div>
            </div>
          );
        })}
      </div>
      )}

      <RoleFormModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        editingRole={editingRole}
      />
      <ConfirmationDialog />
    </div>
  );
}


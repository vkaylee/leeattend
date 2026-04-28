import React, { useState } from "react";
import PageHeader from "@/components/ui/PageHeader";
import { 
  KeyRound,
  Save,
  Check,
  Info,
  SlidersHorizontal,
  ChevronLeft,
  ChevronRight
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useWorkspaceRoles, useWorkspacePermissionsMatrix } from "@/hooks/use-workspace";
import { updateRolePermissions, getCurrentWorkspaceId } from "@/lib/api";
import { useEffect, useRef } from "react";
import PermissionGate from "@/components/common/PermissionGate";
import { 
  DropdownMenu, 
  DropdownMenuTrigger, 
  DropdownMenuContent, 
  DropdownMenuCheckboxItem,
  DropdownMenuLabel,
  DropdownMenuSeparator
} from "@/components/ui/dropdown-menu";
import { useMyPermissions } from "@/hooks/use-permissions";
import { permissionGroups } from "@/lib/permission-groups";
import { useTranslation } from "react-i18next";

export default function WorkspacePermissionsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { roles, isLoading: isRolesLoading } = useWorkspaceRoles(wsId || undefined);
  const { matrix, isLoading: isMatrixLoading, refetch } = useWorkspacePermissionsMatrix(wsId || undefined);
  const { hasPermission } = useMyPermissions();
  const hasManagePermission = hasPermission("members:roles:manage");

  const [permissions, setPermissions] = useState<Record<string, string[]>>({});
  const [hasChanges, setHasChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [hiddenRoleIds, setHiddenRoleIds] = useState<string[]>([]);

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(true);

  const visibleRoles = roles.filter(r => !hiddenRoleIds.includes(r.id));

  const handleScroll = () => {
    if (scrollContainerRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollContainerRef.current;
      setCanScrollLeft(scrollLeft > 0);
      setCanScrollRight(scrollLeft + clientWidth < scrollWidth - 1);
    }
  };

  useEffect(() => {
    handleScroll();
    window.addEventListener("resize", handleScroll);
    return () => window.removeEventListener("resize", handleScroll);
  }, [visibleRoles.length, roles.length]);

  const scrollBy = (amount: number) => {
    if (scrollContainerRef.current) {
      scrollContainerRef.current.scrollBy({ left: amount, behavior: "smooth" });
    }
  };

  useEffect(() => {
    if (matrix) {
      setPermissions(matrix);
    }
  }, [matrix]);

  const togglePermission = (role: any, permId: string) => {
    if (role.is_system || !hasManagePermission) return; // System permissions cannot be changed

    setPermissions(prev => {
      const rolePerms = prev[role.id] || [];
      const newPerms = rolePerms.includes(permId)
        ? rolePerms.filter(p => p !== permId)
        : [...rolePerms, permId];
      
      setHasChanges(true);
      return { ...prev, [role.id]: newPerms };
    });
  };

  const handleSave = async () => {
    if (!wsId) return;
    setIsSaving(true);
    try {
      // Find which roles have been modified by comparing with matrix
      const modifiedRoles = roles.filter(role => {
        const initial = matrix[role.id] || [];
        const current = permissions[role.id] || [];
        // simple array equality
        return initial.length !== current.length || !initial.every(p => current.includes(p));
      });

      await Promise.all(modifiedRoles.map(role => 
        updateRolePermissions(wsId, role.id, permissions[role.id] || [])
      ));
      
      setHasChanges(false);
      refetch();
    } catch (e) {
      console.error("Failed to save permissions", e);
    } finally {
      setIsSaving(false);
    }
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

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className="flex items-center gap-2 px-4 py-2.5 rounded-2xl font-bold text-sm bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors shadow-sm">
                <SlidersHorizontal size={16} className="text-slate-500" />
                Columns
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <DropdownMenuLabel>Visible Roles</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {roles.map(role => (
                  <DropdownMenuCheckboxItem
                  key={role.id}
                  checked={!hiddenRoleIds.includes(role.id)}
                  onCheckedChange={(checked) => {
                    setHiddenRoleIds(prev => 
                      checked ? prev.filter(id => id !== role.id) : [...prev, role.id]
                    );
                  }}
                >
                  {t(`roles.${role.name}`, { defaultValue: role.name })}
                  {role.is_system && <span className="ml-2 text-[9px] text-slate-400 font-normal uppercase tracking-widest bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded">System</span>}
                </DropdownMenuCheckboxItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          <PermissionGate permission="members:roles:manage">
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
          </PermissionGate>
        </div>
      </div>
      
      <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden">
        {isRolesLoading || isMatrixLoading ? (
          <div className="flex justify-center p-20">
            <div className="w-10 h-10 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin"></div>
          </div>
        ) : (
        <div className="relative group/table">
          {/* Scroll Visual Cues */}
          <div className={cn("absolute left-[280px] top-0 bottom-0 w-12 bg-gradient-to-r from-slate-200/50 dark:from-slate-950/50 to-transparent z-10 pointer-events-none transition-opacity duration-300", canScrollLeft ? "opacity-100" : "opacity-0")} />
          <div className={cn("absolute right-0 top-0 bottom-0 w-12 bg-gradient-to-l from-slate-200/50 dark:from-slate-950/50 to-transparent z-10 pointer-events-none transition-opacity duration-300", canScrollRight ? "opacity-100" : "opacity-0")} />
          
          {/* Scroll Buttons - Sticky to viewport */}
          <div className="sticky top-[30vh] z-30 h-0 w-full pointer-events-none">
            <button 
              onClick={() => scrollBy(-300)}
              className={cn("absolute left-[265px] top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto", canScrollLeft ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none")}
            >
              <ChevronLeft size={20} />
            </button>
            
            <button 
              onClick={() => scrollBy(300)}
              className={cn("absolute right-4 top-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 shadow-lg text-slate-700 dark:text-slate-300 rounded-full p-2.5 transition-all hover:scale-110 pointer-events-auto", canScrollRight ? "opacity-0 group-hover/table:opacity-100" : "opacity-0 pointer-events-none")}
            >
              <ChevronRight size={20} />
            </button>
          </div>

          <div 
            ref={scrollContainerRef}
            onScroll={handleScroll}
            className="overflow-x-auto custom-scrollbar relative z-0"
          >
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
                {visibleRoles.map(role => (
                  <th key={role.id} className="bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 p-6 text-center min-w-[140px]">
                    <div className="flex flex-col items-center justify-center gap-2">
                      <h4 className="font-bold text-sm text-slate-800 dark:text-slate-200 truncate w-full px-2" title={t(`roles.${role.name}`, { defaultValue: role.name })}>
                        {t(`roles.${role.name}`, { defaultValue: role.name })}
                      </h4>
                      {role.is_system ? (
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
                      colSpan={visibleRoles.length + 1} 
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
                      
                      {visibleRoles.map(role => {
                        const isChecked = permissions[role.id]?.includes(perm.id);
                        const isSystem = role.is_system;
                        
                        return (
                          <td key={`${role.id}-${perm.id}`} className="border-b border-slate-100 dark:border-slate-800 p-4 text-center">
                            <label className={cn(
                              "relative inline-flex items-center justify-center w-6 h-6 rounded-lg border-2 transition-all",
                              isSystem || !hasManagePermission ? "cursor-not-allowed opacity-60" : "cursor-pointer hover:border-indigo-400",
                              isChecked 
                                ? "bg-indigo-500 border-indigo-500 text-white" 
                                : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                            )}>
                              <input 
                                type="checkbox" 
                                className="sr-only"
                                checked={!!isChecked}
                                disabled={isSystem || !hasManagePermission}
                                onChange={() => togglePermission(role, perm.id)}
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
        </div>
        )}
        
        <div className="bg-slate-50 dark:bg-slate-800/50 p-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-center gap-2">
          <Info className="w-4 h-4 text-slate-400" />
          <p className="text-xs font-medium text-slate-500">System roles (like Admin) have fixed permissions that cannot be modified.</p>
        </div>
      </div>
    </div>
  );
}

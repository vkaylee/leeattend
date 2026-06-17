import { Plus, Building2, LayoutGrid, List, Search, MoreHorizontal, Calculator, Pencil, Trash2 } from "lucide-react";
import React from "react";

import { DepartmentCard } from "@/components/features/departments/department-card";
import { DepartmentConfigModal } from "@/components/features/departments/department-config-modal";
import { DepartmentFormModal } from "@/components/features/departments/department-form-modal";
import { DepartmentDetailDrawer } from "@/components/features/departments/department-detail-drawer";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";

import { useDepartmentsPage } from "./departments/hooks/use-departments-page";

export default function DepartmentsPage() {
  const {
    t,
    wsId,
    isModalOpen,
    setIsModalOpen,
    isConfigModalOpen,
    setIsConfigModalOpen,
    editingDept,
    setEditingDept,
    selectedDeptForDetail,
    setSelectedDeptForDetail,
    viewMode,
    setViewMode,
    searchQuery,
    setSearchQuery,
    selectedManagerId,
    setSelectedManagerId,
    ConfirmationDialog,
    departments,
    isLoading,
    employees,
    deleteMutation,
    handleDelete,
    managerOptions,
    filteredDepartments,
  } = useDepartmentsPage();

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <PageHeader
        actions={
          <Button data-testid="add-dept-btn" icon={<Plus size={18} />} onClick={() => { setEditingDept(null); setIsModalOpen(true); }} variant="ghost">{t("departments.add_btn").toUpperCase()}</Button>
        }
        data-testid="departments-title"
        icon={<Building2 size={24} />}
        subtitle={t("departments.subtitle")}
        title={t("departments.title")}
      />

      {/* Search and Filter Toolbar */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 pb-6 border-b border-slate-100 dark:border-slate-800">
        <div className="flex flex-1 flex-col md:flex-row items-stretch md:items-center gap-4">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" size={18} />
            <input
              type="text"
              placeholder={t("departments.search_placeholder")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-800 rounded-2xl text-sm font-semibold tracking-tight text-slate-700 dark:text-slate-200 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
            />
          </div>
          
          <div className="w-full md:w-72">
            <Combobox
              className="bg-slate-50 dark:bg-slate-850 border-slate-200 dark:border-slate-800 h-14"
              options={managerOptions}
              value={selectedManagerId}
              onChange={setSelectedManagerId}
              placeholder={t("departments.filter_manager_label")}
            />
          </div>
        </div>

        {/* View Mode Toggle Buttons */}
        <div className="flex items-center gap-3 border border-slate-200 dark:border-slate-800 p-1.5 rounded-2xl bg-slate-50/50 dark:bg-slate-800/10">
          <Button
            variant={viewMode === "grid" ? "secondary" : "ghost"}
            size="sm"
            icon={<LayoutGrid size={16} />}
            onClick={() => setViewMode("grid")}
            className="h-10 px-4 rounded-xl font-bold"
          >
            {t("departments.view_grid")}
          </Button>
          <Button
            variant={viewMode === "table" ? "secondary" : "ghost"}
            size="sm"
            icon={<List size={16} />}
            onClick={() => setViewMode("table")}
            className="h-10 px-4 rounded-xl font-bold"
          >
            {t("departments.view_list")}
          </Button>
        </div>
      </div>

      {/* Content Rendering based on View Mode */}
      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {Array.from({ length: 3 }).map((_, index) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" key={index} />
          ))}
        </div>
      ) : departments.length === 0 ? (
        <EmptyState
          description={t("departments.messages.empty_desc")}
          icon={<Building2 size={40} />}
          title={t("departments.messages.no_departments")}
        />
      ) : filteredDepartments.length === 0 ? (
        <EmptyState
          description={t("departments.messages.empty_desc")}
          icon={<Search size={40} />}
          title={t("departments.messages.no_results")}
          action={
            <Button
              variant="secondary"
              onClick={() => {
                setSearchQuery("");
                setSelectedManagerId("all");
              }}
            >
              {t("common.reset", { defaultValue: "Reset" }).toUpperCase()}
            </Button>
          }
        />
      ) : viewMode === "grid" ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {filteredDepartments.map((dept) => (
            <DepartmentCard
              deleteMutationMutate={deleteMutation.mutate}
              dept={dept}
              employees={employees}
              key={dept.id}
              onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
              onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
              onClick={() => setSelectedDeptForDetail(dept)}
              t={t}
            />
          ))}
        </div>
      ) : (
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in duration-500">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50 dark:bg-slate-800/10">
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("departments.columns.name")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("departments.columns.desc")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">{t("departments.columns.manager")}</th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 text-center">{t("departments.columns.employees")}</th>
                <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-right"></th>
              </tr>
            </thead>
            <tbody>
              {filteredDepartments.map((dept) => {
                const manager = employees.find((e) => e.id === dept.manager_id);
                return (
                  <tr
                    key={dept.id}
                    onClick={() => setSelectedDeptForDetail(dept)}
                    className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 cursor-pointer transition-colors group"
                  >
                    <td className="px-8 py-6">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950/50 flex items-center justify-center text-primary-500 dark:text-primary-400 shadow-inner group-hover:scale-110 transition-transform">
                          <Building2 size={16} />
                        </div>
                        <span className="font-bold text-slate-800 dark:text-slate-200 text-xs uppercase tracking-tight">
                          {dept.name}
                        </span>
                      </div>
                    </td>
                    <td className="px-8 py-6 max-w-xs truncate text-slate-500 dark:text-slate-400 text-xs font-semibold uppercase tracking-tight opacity-65 leading-relaxed italic">
                      {dept.description || t("common.no_description")}
                    </td>
                    <td className="px-8 py-6 text-slate-600 dark:text-slate-300 text-xs font-bold uppercase tracking-tight">
                      {manager ? manager.full_name : t("departments.not_assigned").toUpperCase()}
                    </td>
                    <td className="px-8 py-6 text-center text-slate-700 dark:text-slate-300 text-xs font-bold">
                      {dept.active_employee_count}
                    </td>
                    <td className="px-8 py-6 text-right" onClick={(e) => e.stopPropagation()}>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            className="hover:text-primary-600 dark:hover:text-primary-400"
                            icon={<MoreHorizontal size={18} />}
                            size="icon"
                            variant="outline"
                            aria-label={t("departments.columns.actions")}
                          />
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => { setEditingDept(dept); setIsConfigModalOpen(true); }}>
                            <Calculator className="mr-2 h-4 w-4" />
                            {t("payroll.settings.title")}
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => { setEditingDept(dept); setIsModalOpen(true); }}>
                            <Pencil className="mr-2 h-4 w-4" />
                            {t("common.edit")}
                          </DropdownMenuItem>
                          <DropdownMenuItem className="text-rose-600 dark:text-rose-400" onClick={() => { void handleDelete(dept.id); }}>
                            <Trash2 className="mr-2 h-4 w-4" />
                            {t("common.delete")}
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {/* Slide-over Detail Drawer */}
      <DepartmentDetailDrawer
        dept={selectedDeptForDetail}
        employees={employees}
        isOpen={Boolean(selectedDeptForDetail)}
        onClose={() => setSelectedDeptForDetail(null)}
        onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
        onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
        onDelete={(d) => { setSelectedDeptForDetail(null); void handleDelete(d.id); }}
        t={t}
        wsId={wsId}
      />

      <DepartmentConfigModal
        editingDept={editingDept}
        isOpen={isConfigModalOpen}
        onClose={() => { setIsConfigModalOpen(false); }}
        t={t}
        wsId={wsId}
      />

      <DepartmentFormModal
        departments={departments}
        editingDept={editingDept}
        employees={employees}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); setEditingDept(null); }}
        t={t}
        wsId={wsId}
      />

      <ConfirmationDialog />
    </div>
  );
}
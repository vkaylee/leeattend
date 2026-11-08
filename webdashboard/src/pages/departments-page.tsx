/* eslint-disable max-lines-per-function */
import { Plus, Building2, LayoutGrid, List, Search, MoreHorizontal, Calculator, Pencil, Trash2 } from "lucide-react";
import React from "react";

import { DepartmentCard } from "@/components/features/departments/department-card";
import { DepartmentConfigModal } from "@/components/features/departments/department-config-modal";
import { DepartmentDetailDrawer } from "@/components/features/departments/department-detail-drawer";
import { DepartmentFormModal } from "@/components/features/departments/department-form-modal";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import ResponsiveTable from "@/components/ui/responsive-table";

import { useDepartmentsPage } from "./departments/hooks/use-departments-page";

export default function DepartmentsPage() {
  const {
    t,
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


  return (
    <PageContainer>
      <PageHeader
        actions={
          <Button data-testid="add-dept-btn" icon={<Plus size={18} />} onClick={() => { setEditingDept(null); setIsModalOpen(true); }} variant="primary">{t("departments.add_btn")}</Button>
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
              className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-850 border border-slate-200 dark:border-slate-800 rounded-2xl text-sm font-semibold tracking-tight text-slate-700 dark:text-slate-200 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
              onChange={(e) => { setSearchQuery(e.target.value); }}
              placeholder={t("departments.search_placeholder")}
              type="text"
              value={searchQuery}
            />
          </div>

          <div className="w-full md:w-72">
            <Combobox
              className="bg-slate-50 dark:bg-slate-850 border-slate-200 dark:border-slate-800 h-14"
              onChange={setSelectedManagerId}
              options={managerOptions}
              placeholder={t("departments.filter_manager_label")}
              value={selectedManagerId}
            />
          </div>
        </div>

        {/* View Mode Toggle Buttons */}
        <div className="flex items-center gap-3 border border-slate-200 dark:border-slate-800 p-1.5 rounded-2xl bg-slate-50/50 dark:bg-slate-800/10">
          <Button
            className="h-10 px-4 rounded-xl font-bold"
            icon={<LayoutGrid size={16} />}
            onClick={() => { setViewMode("grid"); }}
            size="sm"
            variant={viewMode === "grid" ? "secondary" : "ghost"}
          >
            {t("departments.view_grid")}
          </Button>
          <Button
            className="h-10 px-4 rounded-xl font-bold"
            icon={<List size={16} />}
            onClick={() => { setViewMode("table"); }}
            size="sm"
            variant={viewMode === "table" ? "secondary" : "ghost"}
          >
            {t("departments.view_list")}
          </Button>
        </div>
      </div>

      {/* Content Rendering based on View Mode */}
      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {Array.from({ length: 3 }).map((_, index) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-10 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-80" key={index} />
          ))}
        </div>
      ) : departments.length === 0 ? (
        <EmptyState
          action={
            <Button
              icon={<Plus size={16} />}
              onClick={() => {
                setEditingDept(null);
                setIsModalOpen(true);
              }}
              variant="primary"
            >
              {t("departments.add_btn")}
            </Button>
          }
          description={t("departments.messages.empty_desc")}
          icon={<Building2 size={40} />}
          title={t("departments.messages.no_departments")}
        />
      ) : filteredDepartments.length === 0 ? (
        <EmptyState
          action={
            <Button
              onClick={() => {
                setSearchQuery("");
                setSelectedManagerId("all");
              }}
              variant="secondary"
            >
              {t("common.reset", { defaultValue: "Reset" })}
            </Button>
          }
          description={t("departments.messages.empty_desc")}
          icon={<Search size={40} />}
          title={t("departments.messages.no_results")}
        />
      ) : viewMode === "grid" ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {filteredDepartments.map((dept) => (
            <DepartmentCard
              deleteMutationMutate={deleteMutation.mutate}
              dept={dept}
              employees={employees}
              key={dept.id}
              onClick={() => { setSelectedDeptForDetail(dept); }}
              onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
              onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
              t={t}
            />
          ))}
        </div>
      ) : (
        <ResponsiveTable<typeof filteredDepartments[number]>
          data={filteredDepartments}
          emptyState={{
            icon: <Building2 className="h-10 w-10" />,
            title: t("departments.messages.no_results"),
          }}
          getCardKey={(dept) => dept.id}
          renderCard={(dept) => {
            const manager = employees.find((e) => e.id === dept.manager_id);
            return (
              <div
                className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900"
                data-testid={`dept-card-${dept.id}`}
                onClick={() => { setSelectedDeptForDetail(dept); }}
                onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setSelectedDeptForDetail(dept); } }}
                role="button"
                tabIndex={0}
              >
                {/* Header */}
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary-50 dark:bg-primary-950/50 text-primary-500 dark:text-primary-400 shadow-inner">
                      <Building2 size={18} />
                    </div>
                    <div className="min-w-0">
                      <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
                        {dept.name}
                      </div>
                      { }
                      <div className="mt-0.5 truncate text-[10px] font-semibold italic text-slate-400 opacity-70 dark:text-slate-500">
                        {dept.description ?? t("common.no_description")}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Detail grid */}
                <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
                  <div>
                    <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("departments.columns.manager")}</div>
                    <div className="mt-1 text-xs font-medium text-slate-600 dark:text-slate-300">
                      {manager ? manager.full_name : t("departments.not_assigned")}
                    </div>
                  </div>
                  <div>
                    <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("departments.columns.employees")}</div>
                    <div className="mt-1 text-xs font-bold text-slate-700 dark:text-slate-300">
                      {dept.active_employee_count}
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className="mt-4 flex items-center justify-end gap-1 border-t border-slate-100 pt-4 dark:border-slate-800">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        aria-label={t("departments.columns.actions")}
                        className="hover:text-primary-600 dark:hover:text-primary-400"
                        icon={<MoreHorizontal size={18} />}
                        size="icon"
                        variant="outline"
                      />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={(e) => { e.stopPropagation(); setEditingDept(dept); setIsConfigModalOpen(true); }}>
                        <Calculator className="mr-2 h-4 w-4" />
                        {t("payroll.settings.title")}
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={(e) => { e.stopPropagation(); setEditingDept(dept); setIsModalOpen(true); }}>
                        <Pencil className="mr-2 h-4 w-4" />
                        {t("common.edit")}
                      </DropdownMenuItem>
                      <DropdownMenuItem className="text-rose-600 dark:text-rose-400" onClick={(e) => { e.stopPropagation(); void handleDelete(dept.id); }}>
                        <Trash2 className="mr-2 h-4 w-4" />
                        {t("common.delete")}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            );
          }}
          renderDesktopTable={() => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in duration-500">
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
                        className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 cursor-pointer transition-colors group"
                        key={dept.id}
                        onClick={() => { setSelectedDeptForDetail(dept); }}
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
                        { }
                        <td className="px-8 py-6 max-w-xs truncate text-slate-500 dark:text-slate-400 text-xs font-semibold uppercase tracking-tight opacity-65 leading-relaxed italic">
                          {dept.description ?? t("common.no_description")}
                        </td>
                        <td className="px-8 py-6 text-slate-600 dark:text-slate-300 text-xs font-medium">
                          {manager ? manager.full_name : t("departments.not_assigned")}
                        </td>
                        <td className="px-8 py-6 text-center text-slate-700 dark:text-slate-300 text-xs font-bold">
                          {dept.active_employee_count}
                        </td>
                        <td className="px-8 py-6 text-right" onClick={(e) => { e.stopPropagation(); }}>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button
                                aria-label={t("departments.columns.actions")}
                                className="hover:text-primary-600 dark:hover:text-primary-400"
                                icon={<MoreHorizontal size={18} />}
                                size="icon"
                                variant="outline"
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
        />
      )}

      {/* Slide-over Detail Drawer */}
      <DepartmentDetailDrawer dept={selectedDeptForDetail}
      employees={employees}
      isOpen={Boolean(selectedDeptForDetail)}
      onClose={() => { setSelectedDeptForDetail(null); }}
      onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
      onDelete={(d) => { setSelectedDeptForDetail(null); void handleDelete(d.id); }}
      onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
      t={t}  />

      <DepartmentConfigModal editingDept={editingDept}
      isOpen={isConfigModalOpen}
      onClose={() => { setIsConfigModalOpen(false); }}
      t={t}  />

      <DepartmentFormModal departments={departments}
      editingDept={editingDept}
      employees={employees}
      isOpen={isModalOpen}
      onClose={() => { setIsModalOpen(false); setEditingDept(null); }}
      t={t}  />

      <ConfirmationDialog />
    </PageContainer>
  );
}
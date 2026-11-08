import type { TFunction } from "i18next";

import { LayoutGrid, UserCircle } from "lucide-react";
import React, { useMemo } from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField, { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";

import { useDepartmentForm } from "./hooks/use-department-form";

export interface DepartmentFormModalProps {
  departments: Department[];
  editingDept: Department | null;
  employees: Employee[];
  isOpen: boolean;
  onClose: () => void;
  t: TFunction;
}

export function DepartmentFormModal({ 
  departments, 
  editingDept, 
  isOpen, 
  onClose,employees, 
  t 
}: DepartmentFormModalProps) {
  const { 
    selectedManagerId, 
    setSelectedManagerId, 
    selectedParentId,
    setSelectedParentId,
    handleSubmit,
    isLoading 
  } = useDepartmentForm(editingDept, onClose, t);

  const sortedEmployees = useMemo(() => {
    if (!editingDept) return employees;
    const thisDept = employees.filter(e => e.department_name === editingDept.name);
    const others = employees.filter(e => e.department_name !== editingDept.name);
    return [...thisDept, ...others];
  }, [employees, editingDept]);

  // Load parent map from localStorage to filter out descendants and prevent cyclic parenting
  const parentMap = useMemo<Record<string, string>>(() => {
    try {
      const saved = localStorage.getItem("leeattend_dept_parents");
      if (!saved) return {};
      const parsed = JSON.parse(saved) as unknown;
      if (parsed && typeof parsed === "object") {
        return parsed as Record<string, string>;
      }
      return {};
    } catch { /* localStorage unavailable or corrupt — return empty fallback */
      return {};
    }
  }, []);

  const validParents = useMemo(() => {
    return departments.filter(d => {
      // Cannot be self
      if (d.id === editingDept?.id) return false;
      // Cannot be a descendant (prevent cycles)
      if (editingDept) {
        let currentParent = parentMap[d.id];
        while (currentParent) {
          if (currentParent === editingDept.id) return false;
          currentParent = parentMap[currentParent];
        }
      }
      return true;
    });
  }, [departments, editingDept, parentMap]);

  return (
    <Modal icon={<LayoutGrid size={24} />} isOpen={isOpen} maxWidth="lg" onClose={onClose} subtitle={t("departments.modal.subtitle")} title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")}>
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <FormField defaultValue={editingDept?.name} id="dept-name" label={t("departments.form.name_label")} name="name" placeholder={t("departments.form.name_placeholder")} required />
        <FormField defaultValue={editingDept?.description ?? undefined} id="dept-desc" label={t("common.description")} name="description" placeholder={t("departments.form.desc_placeholder")} rows={4} type="textarea" />
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {/* Manager Select */}
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE} htmlFor="dept-manager">{t("departments.manager_label")}</label>
            <Combobox 
              className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" 
              data-testid="dept-manager" 
              id="dept-manager" 
              onChange={setSelectedManagerId} 
              options={sortedEmployees.map(e => ({ 
                value: e.id, 
                label: e.full_name.toUpperCase(), 
                description: e.department_name ? `${e.employee_id_number} • ${e.department_name}` : e.employee_id_number, 
                icon: <UserCircle size={14} /> 
              }))} 
              placeholder={t("departments.form.manager_placeholder")} 
              value={selectedManagerId} 
            />
          </div>

          {/* Parent Department Select */}
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE} htmlFor="dept-parent">{t("departments.parent_label", { defaultValue: "PARENT DEPARTMENT" })}</label>
            <Combobox 
              className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" 
              data-testid="dept-parent" 
              id="dept-parent" 
              onChange={setSelectedParentId} 
              options={[
                { value: "", label: t("common.none", { defaultValue: "NONE" }).toUpperCase() },
                ...validParents.map(d => ({ 
                  value: d.id, 
                  label: d.name.toUpperCase(), 
                  description: d.description ?? undefined
                }))
              ]} 
              placeholder={t("departments.form.parent_placeholder", { defaultValue: "Select parent..." })} 
              value={selectedParentId} 
            />
          </div>
        </div>

        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button data-testid="save-dept-btn" loading={isLoading} size="lg" type="submit" variant="secondary">{editingDept ? t("common.update").toUpperCase() : t("departments.modal.submit_btn").toUpperCase()}</Button>
        </div>
      </form>
    </Modal>
  );
}

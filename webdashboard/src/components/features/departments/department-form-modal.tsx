import type { TFunction } from "i18next";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { LayoutGrid, UserCircle, CalendarDays } from "lucide-react";
import React, { useState } from "react";

import type { Department, User, WorkingPattern } from "@/lib/api";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField, { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { createDepartment, updateDepartment } from "@/lib/api";

export interface DepartmentFormModalProps {
  editingDept: Department | null;
  isOpen: boolean;
  onClose: () => void;
  patterns: WorkingPattern[];
  t: TFunction;
  users: User[];
  wsId: string;
}

export function DepartmentFormModal({ editingDept, isOpen, onClose, wsId, users, patterns, t }: DepartmentFormModalProps) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const [selectedManagerId, setSelectedManagerId] = useState<string>("");
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");

  const [prevDeptId, setPrevDeptId] = useState<string | null>(editingDept?.id ?? null);
  if (editingDept?.id !== prevDeptId) {
    setSelectedManagerId(editingDept?.manager_id ?? "");
    setSelectedPatternId(editingDept?.default_working_pattern_id ?? "");
    setPrevDeptId(editingDept?.id ?? null);
  }

  const createMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => createDepartment(wsId, data),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
      onClose(); 
      toast.success(t("common.success")); 
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => {
      if (!editingDept) throw new Error("Editing department is required for update");
      return updateDepartment(wsId, editingDept.id, data);
    },
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
      onClose(); 
      toast.success(t("common.success")); 
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault(); 
    const formData = new FormData(e.currentTarget);
    const data = { 
      name: formData.get("name") as string, 
      description: formData.get("description") as string, 
      manager_id: selectedManagerId || null, 
      default_working_pattern_id: selectedPatternId || null 
    };
    if (editingDept) { 
      updateMutation.mutate(data); 
    } else { 
      createMutation.mutate(data); 
    }
  };

  return (
    <Modal icon={<LayoutGrid size={24} />} isOpen={isOpen} maxWidth="lg" onClose={onClose} subtitle={t("departments.modal.subtitle")} title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")}>
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <FormField defaultValue={editingDept?.name} id="dept-name" label={t("departments.form.name_label")} name="name" placeholder={t("departments.form.name_placeholder")} required />
        <FormField defaultValue={editingDept?.description ?? undefined} id="dept-desc" label={t("common.description")} name="description" placeholder={t("departments.form.desc_placeholder")} rows={4} type="textarea" />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="space-y-3 px-1"><label className={LABEL_BASE} htmlFor="dept-manager">{t("departments.manager_label")}</label><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" id="dept-manager" onChange={setSelectedManagerId} options={users.map(u => ({ value: u.id, label: (u.full_name ?? u.email).toUpperCase(), icon: <UserCircle size={14} /> }))} placeholder={t("departments.form.manager_placeholder")} value={selectedManagerId} /></div>
          <div className="space-y-3 px-1"><label className={LABEL_BASE} htmlFor="dept-pattern">{t("departments.default_pattern_label")}</label><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" data-testid="dept-pattern" id="dept-pattern" onChange={setSelectedPatternId} options={patterns.map(p => ({ value: p.id, label: p.name.toUpperCase(), icon: <CalendarDays size={14} /> }))} placeholder={t("departments.form.pattern_placeholder")} value={selectedPatternId} /></div>
        </div>
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button data-testid="save-dept-btn" size="lg" type="submit" variant="secondary">{editingDept ? t("common.update").toUpperCase() : t("departments.modal.submit_btn").toUpperCase()}</Button>
        </div>
      </form>
    </Modal>
  );
}

import React from "react";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/Button";
import FormField from "@/components/ui/FormField";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";

import { createEmployee, updateEmployee, Employee } from "@/lib/api";

interface EmployeeFormModalProps {
  isOpen: boolean;
  onClose: () => void;
  editingEmployee: Employee | null;
  wsId: string;
  prefillEnrollmentId?: string;
  onSuccess?: () => void;
}

export default function EmployeeFormModal({
  isOpen,
  onClose,
  editingEmployee,
  wsId,
  prefillEnrollmentId = "",
  onSuccess,
}: EmployeeFormModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) =>
      updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      full_name: formData.get("full_name") as string,
      employee_id_number: formData.get("employee_id_number") as string,
      hire_date: formData.get("hire_date") as string,
      is_active: formData.get("is_active") === "on",
    };
    if (editingEmployee) {
      updateMutation.mutate({ id: editingEmployee.id, employee: data });
    } else {
      createMutation.mutate(data);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")}
      subtitle={t("employees.form.subtitle")}
      icon={<UserCircle size={24} />}
      maxWidth="lg"
    >
      <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
        <FormField
          label={t("employees.form.full_name")}
          name="full_name"
          data-testid="employees-form-name-input"
          required
          defaultValue={editingEmployee?.full_name}
        />
        <FormField
          label={t("employees.form.employee_id")}
          name="employee_id_number"
          data-testid="employees-form-id-input"
          required
          defaultValue={editingEmployee?.employee_id_number || prefillEnrollmentId}
          inputClassName="font-mono tracking-widest"
        />
        <FormField
          label={t("employees.form.hire_date")}
          name="hire_date"
          type="date"
          data-testid="employees-form-hire-date-input"
          required
          defaultValue={editingEmployee?.hire_date}
        />
        <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
          <input
            type="checkbox"
            name="is_active"
            id="is_active"
            data-testid="employees-form-active-checkbox"
            defaultChecked={editingEmployee ? editingEmployee.is_active : true}
            className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner"
          />
          <label
            htmlFor="is_active"
            className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors"
          >
            {t("employees.form.is_active")}
          </label>
        </div>
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button variant="outline" size="lg" onClick={onClose} type="button">
            {t("employees.form.cancel").toUpperCase()}
          </Button>
          <Button
            variant="secondary"
            size="lg"
            type="submit"
            data-testid="save-employee-btn"
            loading={createMutation.isPending || updateMutation.isPending}
          >
            {editingEmployee ? t("employees.form.update").toUpperCase() : t("employees.form.save").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

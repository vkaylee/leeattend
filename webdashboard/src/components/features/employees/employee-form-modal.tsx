import { useMutation, useQueryClient } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { WorkspaceMember } from "@/lib/api";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { createEmployee, updateEmployee, fetchWorkspaceMembers, Employee } from "@/lib/api";

interface EmployeeFormModalProps {
  editingEmployee: Employee | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  prefillEnrollmentId?: string;
  wsId: string;
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

  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  const [selectedUserId, setSelectedUserId] = useState(editingEmployee?.user_id ?? "");

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
    setSelectedUserId(editingEmployee?.user_id ?? "");
  }, [editingEmployee]);

  useEffect(() => {
    if (isOpen && wsId) {
      void fetchWorkspaceMembers(wsId).then(setMembers).catch(() => { /* empty */ });
    }
  }, [isOpen, wsId]);

  const createMutation = useMutation({
    mutationFn: (data: Partial<Employee>) => createEmployee(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: Partial<Employee> }) =>
      updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data: Partial<Employee> = {
      full_name: formData.get("full_name") as string,
      employee_id_number: formData.get("employee_id_number") as string,
      hire_date: formData.get("hire_date") as string,
      is_active: formData.get("is_active") === "on",
    };
    if (selectedUserId) {
      data.user_id = selectedUserId;
    }
    if (editingEmployee) {
      updateMutation.mutate({ id: editingEmployee.id, employee: data });
    } else {
      createMutation.mutate(data);
    }
  };

  return (
    <Modal
      icon={<UserCircle size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("employees.form.subtitle")}
      title={editingEmployee ? t("employees.form.edit_title") : t("employees.form.add_title")}
    >
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <FormField
          data-testid="employees-form-name-input"
          defaultValue={editingEmployee?.full_name}
          label={t("employees.form.full_name")}
          name="full_name"
          required
        />
        <FormField
          data-testid="employees-form-id-input"
          defaultValue={editingEmployee?.employee_id_number ?? prefillEnrollmentId}
          inputClassName="font-mono tracking-widest"
          label={t("employees.form.employee_id")}
          name="employee_id_number"
          required
        />
        <FormField
          data-testid="employees-form-hire-date-input"
          defaultValue={editingEmployee?.hire_date}
          label={t("employees.form.hire_date")}
          name="hire_date"
          required
          type="date"
        />
        <div className="space-y-3 px-1">
          <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
            {t("employees.form.link_user")}
          </label>
          <Combobox
            clearOnEscape
            data-testid="employees-form-user-combobox"
            onChange={setSelectedUserId}
            options={members.map(m => ({
              value: m.user_id,
              label: m.full_name ?? m.email,
              description: m.email,
            }))}
            placeholder={t("employees.form.select_user")}
            value={selectedUserId}
          />
        </div>
        <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
          <input
            className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner"
            data-testid="employees-form-active-checkbox"
            defaultChecked={editingEmployee ? editingEmployee.is_active : true}
            id="is_active"
            name="is_active"
            type="checkbox"
          />
          <label
            className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors"
            htmlFor="is_active"
          >
            {t("employees.form.is_active")}
          </label>
        </div>
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" type="button" variant="outline">
            {t("employees.form.cancel").toUpperCase()}
          </Button>
          <Button
            data-testid="save-employee-btn"
            loading={createMutation.isPending || updateMutation.isPending}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {editingEmployee ? t("employees.form.update").toUpperCase() : t("employees.form.save").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

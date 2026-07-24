import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { WorkspaceMember } from "@/lib/api";

import {
  createEmployee, updateEmployee, fetchWorkspaceMembers,
  Employee, fetchEmployers, fetchEmployeeEmployments,
  updateEmployeeEmployment, Employment,
  type CreateEmployeeRequest,
  type UpdateEmployeeRequest,
} from "@/lib/api";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import EmployeeFormInputs from "./employee-form-inputs";

interface EmployeeFormModalProps {
  editingEmployee: Employee | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  prefillEnrollmentId?: string;
  wsId: string;
}

function useEmployeeMutations(wsId: string, onClose: () => void, onSuccess?: () => void) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const createMutation = useMutation({
    mutationFn: (data: CreateEmployeeRequest) => createEmployee(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: { id: string; employee: UpdateEmployeeRequest }) =>
      updateEmployee(wsId, data.id, data.employee),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  return { createMutation, updateMutation };
}

import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";

function useFormDefaults(editingEmployee: Employee | null, activeLocationId: string, activeEmployerId: string) {
  const externalUserId = editingEmployee?.user_id ?? "";
  const externalLocId = editingEmployee ? activeLocationId : "";
  const externalEmpId = editingEmployee ? activeEmployerId : "";

  const [selectedUserId, setSelectedUserId] = useDerivedStateFrom(externalUserId);
  const [selectedLocationId, setSelectedLocationId] = useDerivedStateFrom(externalLocId);
  const [selectedEmployerId, setSelectedEmployerId] = useDerivedStateFrom(externalEmpId);

  return {
    selectedUserId, setSelectedUserId,
    selectedLocationId, setSelectedLocationId,
    selectedEmployerId, setSelectedEmployerId,
  };
}

/**
 * EmployeeFormModal — create or edit an employee.
 *
 * The component has been decomposed into useEmployeeMutations, useFormDefaults,
 * and EmployeeFormInputs. The remaining branches are genuine business logic
 * (create vs update, optional employment mutation).
 */
// eslint-disable-next-line complexity -- JSX composition + query wiring; logic already in useEmployeeMutations / useFormDefaults
export default function EmployeeFormModal({
  isOpen, onClose, editingEmployee, wsId, prefillEnrollmentId, onSuccess,
}: EmployeeFormModalProps) {
  const { t } = useTranslation();
  const { createMutation, updateMutation } = useEmployeeMutations(wsId, onClose, onSuccess);
  const { data: locations = [] } = useLocations(wsId);
  const { data: employers = [] } = useQuery({
    queryKey: ["employers", wsId],
    queryFn: () => fetchEmployers(wsId),
    enabled: !!wsId,
  });

  const { data: employments = [] } = useQuery({
    queryKey: ["employee-employments", wsId, editingEmployee?.id],
    queryFn: () => {
      if (!editingEmployee?.id) throw new Error("editingEmployee.id required");
      return fetchEmployeeEmployments(wsId, editingEmployee.id);
    },
    enabled: !!wsId && !!editingEmployee?.id && isOpen,
  });
  const active: Employment | undefined = employments.find(e => e.status === "active");
  const activeLocationId = active?.primary_location_id ?? "";
  const activeEmployerId = active?.employer_id ?? "";

  const defaults = useFormDefaults(editingEmployee, activeLocationId, activeEmployerId);

  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  useEffect(() => {
    if (isOpen && wsId) {
      const loadMembers = async () => {
        try {
          const data = await fetchWorkspaceMembers(wsId);
          setMembers(data);
        } catch {
          /* empty */
        }
      };
      void loadMembers();
    }
  }, [isOpen, wsId]);

  async function syncEmploymentIfChanged(employee: Employee) {
    if (defaults.selectedLocationId !== activeLocationId || defaults.selectedEmployerId !== activeEmployerId) {
      await updateEmployeeEmployment(wsId, employee.id, active?.id ?? "", {
        primary_location_id: defaults.selectedLocationId || null,
        employer_id: defaults.selectedEmployerId || undefined,
      });
    }
  }

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);

    if (editingEmployee) {
      const payload: UpdateEmployeeRequest = {
        full_name: String(fd.get("full_name") ?? ""),
        employee_id_number: String(fd.get("employee_id_number") ?? ""),
        user_id: defaults.selectedUserId || undefined,
      };
      await updateMutation.mutateAsync({ id: editingEmployee.id, employee: payload });
      await syncEmploymentIfChanged(editingEmployee);
    } else {
      const payload: CreateEmployeeRequest = {
        full_name: String(fd.get("full_name") ?? ""),
        employee_id_number: String(fd.get("employee_id_number") ?? ""),
        hire_date: String(fd.get("hire_date") ?? ""),
        employer_id: defaults.selectedEmployerId || "",
        primary_location_id: defaults.selectedLocationId || undefined,
        user_id: defaults.selectedUserId || undefined,
      };
      createMutation.mutate(payload);
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
        <EmployeeFormInputs
          activeEmployment={active}
          editingEmployee={editingEmployee}
          employers={employers}
          locations={locations}
          members={members}
          prefillEnrollmentId={prefillEnrollmentId ?? ""}
          selectedEmployerId={defaults.selectedEmployerId}
          selectedLocationId={defaults.selectedLocationId}
          selectedUserId={defaults.selectedUserId}
          setSelectedEmployerId={defaults.setSelectedEmployerId}
          setSelectedLocationId={defaults.setSelectedLocationId}
          setSelectedUserId={defaults.setSelectedUserId}
          t={t}
        />
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

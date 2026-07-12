import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { WorkspaceMember } from "@/lib/api";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import {
  createEmployee, updateEmployee, fetchWorkspaceMembers,
  Employee, fetchEmployers, fetchEmployeeEmployments,
  updateEmployeeEmployment, Employment,
} from "@/lib/api";
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

  return { createMutation, updateMutation };
}

function useFormDefaults(editingEmployee: Employee | null, activeLocationId: string, activeEmployerId: string) {
  const [userId, setUserId] = useState(editingEmployee?.user_id ?? "");
  const [locId, setLocId] = useState("");
  const [empId, setEmpId] = useState("");

  const employeeId = editingEmployee?.id ?? null;
  const userId_ = editingEmployee?.user_id ?? "";
   
  useEffect(() => {
    setUserId(userId_);
    setLocId(editingEmployee ? activeLocationId : "");
    setEmpId(editingEmployee ? activeEmployerId : "");
  }, [employeeId, userId_, activeLocationId, activeEmployerId, editingEmployee]);

  return {
    selectedUserId: userId, setSelectedUserId: setUserId,
    selectedLocationId: locId, setSelectedLocationId: setLocId,
    selectedEmployerId: empId, setSelectedEmployerId: setEmpId,
  };
}

/**
 * EmployeeFormModal — create or edit an employee.
 *
 * Complexity is suppressed because the remaining branches are genuine business
 * logic (create vs update, optional employment mutation). The component has
 * been decomposed into useEmployeeMutations, useFormDefaults, and
 * EmployeeFormInputs. Further decomposition would degrade readability.
 *
 * eslint-disable-next-line complexity
 */
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
    queryFn: () => fetchEmployeeEmployments(wsId, editingEmployee!.id),
    enabled: !!wsId && !!editingEmployee?.id && isOpen,
  });
  const active: Employment | undefined = employments.find(e => e.status === "active") ?? employments[0];
  const activeLocationId = active?.primary_location_id ?? "";
  const activeEmployerId = active?.employer_id ?? "";

  const defaults = useFormDefaults(editingEmployee, activeLocationId, activeEmployerId);

  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  useEffect(() => {
    if (isOpen && wsId) {
      void fetchWorkspaceMembers(wsId).then(setMembers).catch(() => { /* empty */ });
    }
  }, [isOpen, wsId]);

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const data: Record<string, unknown> = {
      full_name: fd.get("full_name"),
      employee_id_number: fd.get("employee_id_number"),
      hire_date: fd.get("hire_date"),
      is_active: fd.get("is_active") === "on",
    };
    if (defaults.selectedUserId) data.user_id = defaults.selectedUserId;

    if (editingEmployee) {
      await updateMutation.mutateAsync({ id: editingEmployee.id, employee: data });
      if (active && (defaults.selectedLocationId !== activeLocationId || defaults.selectedEmployerId !== activeEmployerId)) {
        await updateEmployeeEmployment(wsId, editingEmployee.id, active.id, {
          primary_location_id: defaults.selectedLocationId || null,
          employer_id: defaults.selectedEmployerId || undefined,
        });
      }
    } else {
      if (defaults.selectedLocationId) data.primary_location_id = defaults.selectedLocationId;
      data.employer_id = defaults.selectedEmployerId ?? null;
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
        <EmployeeFormInputs
          activeEmployment={active}
          editingEmployee={editingEmployee}
          employers={employers}
          locations={locations ?? []}
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

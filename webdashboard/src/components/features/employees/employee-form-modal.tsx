import { useQuery } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { OrganizationMember, CreateEmployeeRequest, UpdateEmployeeRequest } from "@/lib/api";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import {
  createEmployee, updateEmployee, fetchOrganizationMembers,
  Employee, fetchEmployers, fetchEmployeeEmployments,
  updateEmployeeEmployment, Employment,
} from "@/lib/api";
import { logError } from "@/lib/logger";
import { employeeKeys } from "@/lib/query-keys";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import EmployeeFormInputs from "./employee-form-inputs";
import { createQueryKey } from "@/lib/query-keys";


interface EmployeeFormModalProps {
  editingEmployee: Employee | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  prefillEnrollmentId?: string;
}

function useEmployeeMutations(onClose: () => void, onSuccess?: () => void) {
  const { t } = useTranslation();
  const toast = useToast();

  const createMutation = useOrganizationMutation<
    CreateEmployeeRequest,
    Awaited<ReturnType<typeof createEmployee>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: (data) => createEmployee(data),
    onSuccess: () => {
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const updateMutation = useOrganizationMutation<
    { id: string; employee: UpdateEmployeeRequest },
    Awaited<ReturnType<typeof updateEmployee>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: (data) => updateEmployee(data.id, data.employee),
    onSuccess: () => {
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  return { createMutation, updateMutation };
}

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
  isOpen, onClose, editingEmployee,prefillEnrollmentId, onSuccess,
}: EmployeeFormModalProps) {
  const { t } = useTranslation();
  const { createMutation, updateMutation } = useEmployeeMutations(onClose, onSuccess);
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { data: employers } = useQuery({ queryKey: createQueryKey("employers"), queryFn: () => fetchEmployers(),
 });

  const { data: employments = [] } = useQuery({ queryKey: createQueryKey("employee-employments", editingEmployee?.id), queryFn: () => {
    if (!editingEmployee?.id) throw new Error("editingEmployee.id required");
    return fetchEmployeeEmployments(editingEmployee.id);
  },
  enabled: !!editingEmployee?.id && isOpen, });
  const active: Employment | undefined = employments.find(e => e.status === "active");
  const activeLocationId = active?.primary_location_id ?? "";
  const activeEmployerId = active?.employer_id ?? "";

  const defaults = useFormDefaults(editingEmployee, activeLocationId, activeEmployerId);

  const [members, setMembers] = useState<OrganizationMember[]>([]);
  useEffect(() => {
    if (isOpen ) {
      const loadMembers = async () => {
        try {
          const data = await fetchOrganizationMembers();
          setMembers(data);
        } catch (e: unknown) {
          logError(e, { component: 'EmployeeFormModal.loadMembers' });
          /* proceed with empty member list */
        }
      };
      void loadMembers();
    }
  }, [isOpen]);

  async function syncEmploymentIfChanged(employee: Employee) {
    if (defaults.selectedLocationId !== activeLocationId || defaults.selectedEmployerId !== activeEmployerId) {
      await updateEmployeeEmployment(employee.id, active?.id ?? "", {
        primary_location_id: defaults.selectedLocationId || null,
        employer_id: defaults.selectedEmployerId || undefined,
      });
    }
  }

  const getFormString = (fd: FormData, key: string): string => {
    const val = fd.get(key);
    return typeof val === "string" ? val : "";
  };

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);

    if (editingEmployee) {
      const payload: UpdateEmployeeRequest = {
        full_name: getFormString(fd, "full_name"),
        employee_id_number: getFormString(fd, "employee_id_number"),
        user_id: defaults.selectedUserId || undefined,
      };
      await updateMutation.mutateAsync({ id: editingEmployee.id, employee: payload });
      await syncEmploymentIfChanged(editingEmployee);
    } else {
      const payload: CreateEmployeeRequest = {
        full_name: getFormString(fd, "full_name"),
        employee_id_number: getFormString(fd, "employee_id_number"),
        hire_date: getFormString(fd, "hire_date"),
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
          employers={employers ?? []}
          locations={locations}
          locationsUnavailable={locationsUnavailable}
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

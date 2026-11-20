import { useQuery } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Employee, Employment, UpdateEmployeeRequest } from "@/lib/api";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { fetchEmployeeEmployments, fetchEmployers, updateEmployee, updateEmployeeEmployment } from "@/lib/api";
import { createQueryKey, employeeKeys } from "@/lib/query-keys";
import { useLocations } from "@/pages/settings/hooks/use-locations";

import EmployeeFormInputs from "./employee-form-inputs";

interface EmployeeFormModalProps {
  editingEmployee: Employee;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

function useEmployeeMutation() {
  return useOrganizationMutation<
    { id: string; employee: UpdateEmployeeRequest },
    Awaited<ReturnType<typeof updateEmployee>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: (data) => updateEmployee(data.id, data.employee),
  });
}

function useFormDefaults(activeLocationId: string, activeEmployerId: string) {
  const [selectedLocationId, setSelectedLocationId] = useDerivedStateFrom(activeLocationId);
  const [selectedEmployerId, setSelectedEmployerId] = useDerivedStateFrom(activeEmployerId);

  return {
    selectedLocationId,
    setSelectedLocationId,
    selectedEmployerId,
    setSelectedEmployerId,
  };
}

/** Edit employee profile and employment assignment. Account linking belongs to Members. */
export default function EmployeeFormModal({
  isOpen,
  onClose,
  onSuccess,
  editingEmployee,
}: EmployeeFormModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const updateMutation = useEmployeeMutation();
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { data: employers } = useQuery({
    queryKey: createQueryKey("employers"),
    queryFn: fetchEmployers,
  });
  const { data: employments = [] } = useQuery({
    queryKey: createQueryKey("employee-employments", editingEmployee.id),
    queryFn: () => fetchEmployeeEmployments(editingEmployee.id),
    enabled: isOpen,
  });
  const active: Employment | undefined = employments.find((employment) => employment.status === "active");
  const defaults = useFormDefaults(active?.primary_location_id ?? "", active?.employer_id ?? "");

  async function syncEmploymentIfChanged() {
    if (
      defaults.selectedLocationId !== (active?.primary_location_id ?? "")
      || defaults.selectedEmployerId !== (active?.employer_id ?? "")
    ) {
      await updateEmployeeEmployment(editingEmployee.id, active?.id ?? "", {
        primary_location_id: defaults.selectedLocationId || null,
        employer_id: defaults.selectedEmployerId || undefined,
      });
    }
  }

  const handleSubmit = async (event: React.SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const value = (key: string) => {
      const field = formData.get(key);
      return typeof field === "string" ? field : "";
    };

    try {
      await updateMutation.mutateAsync({
        id: editingEmployee.id,
        employee: {
          full_name: value("full_name"),
          employee_id_number: value("employee_id_number"),
        },
      });
      await syncEmploymentIfChanged();
      onClose();
      onSuccess?.();
      toast.success(t("common.success"));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t("common.error"));
    }
  };

  return (
    <Modal
      icon={<UserCircle size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("employees.form.subtitle")}
      title={t("employees.form.edit_title")}
    >
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <EmployeeFormInputs
          activeEmployment={active}
          editingEmployee={editingEmployee}
          employers={employers ?? []}
          locations={locations ?? []}
          locationsUnavailable={locationsUnavailable}
          members={[]}
          prefillEnrollmentId=""
          selectedEmployerId={defaults.selectedEmployerId}
          selectedLocationId={defaults.selectedLocationId}
          selectedUserId=""
          setSelectedEmployerId={defaults.setSelectedEmployerId}
          setSelectedLocationId={defaults.setSelectedLocationId}
          setSelectedUserId={() => undefined}
          showUserLink={false}
          t={t}
        />
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" type="button" variant="outline">
            {t("employees.form.cancel").toUpperCase()}
          </Button>
          <Button
            data-testid="save-employee-btn"
            loading={updateMutation.isPending}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {t("employees.form.update").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

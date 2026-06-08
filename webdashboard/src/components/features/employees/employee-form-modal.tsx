import type { TFunction } from "i18next";

import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { UserCircle } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { WorkspaceMember } from "@/lib/api";
import type { WorkLocation } from "@/lib/api-generated/models";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { createEmployee, updateEmployee, fetchWorkspaceMembers, Employee, fetchEmployers, Employer, fetchEmployeeEmployments, updateEmployeeEmployment } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";

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


function EmployeeFormInputs({
  editingEmployee,
  prefillEnrollmentId,
  members,
  selectedUserId,
  setSelectedUserId,
  locations,
  selectedLocationId,
  setSelectedLocationId,
  employers,
  selectedEmployerId,
  setSelectedEmployerId,
  t,
}: {
  editingEmployee: Employee | null;
  prefillEnrollmentId: string;
  members: WorkspaceMember[];
  selectedUserId: string;
  setSelectedUserId: (val: string) => void;
  locations: WorkLocation[];
  selectedLocationId: string;
  setSelectedLocationId: (val: string) => void;
  employers: Employer[];
  selectedEmployerId: string;
  setSelectedEmployerId: (val: string) => void;
  t: TFunction;
}) {
  return (
    <>
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
          options={members.map((m) => ({
            value: m.user_id,
            label: m.full_name ?? m.email,
            description: m.email,
          }))}
          placeholder={t("employees.form.select_user")}
          value={selectedUserId}
        />
      </div>
      <div className="space-y-3 px-1">
        <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" htmlFor="employees-form-location-combobox">
          Địa điểm làm việc
        </label>
        <Combobox
          clearOnEscape
          data-testid="employees-form-location-combobox"
          id="employees-form-location-combobox"
          onChange={setSelectedLocationId}
          options={locations.map((loc) => ({
            value: loc.id,
            label: loc.name,
            description: loc.is_headquarters ? "Headquarters" : loc.timezone,
          }))}
          placeholder="Chọn địa điểm làm việc"
          value={selectedLocationId}
        />
      </div>
      <div className="space-y-3 px-1">
        <label className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" htmlFor="employees-form-employer-combobox">
          Pháp nhân trực thuộc
        </label>
        <Combobox
          clearOnEscape
          data-testid="employees-form-employer-combobox"
          id="employees-form-employer-combobox"
          onChange={setSelectedEmployerId}
          options={employers.map((emp) => ({
            value: emp.id,
            label: emp.name,
            description: emp.tax_code ? `MST: ${emp.tax_code}` : undefined,
          }))}
          placeholder="Chọn pháp nhân trực thuộc"
          value={selectedEmployerId}
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
    </>
  );
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
  const { createMutation, updateMutation } = useEmployeeMutations(wsId, onClose, onSuccess);
  const { data: locations = [] } = useLocations(wsId);
  const { data: employers = [] } = useQuery({ queryKey: ["employers", wsId], queryFn: () => fetchEmployers(wsId), enabled: !!wsId });

  const { data: employments = [] } = useQuery({
    queryKey: ["employee-employments", wsId, editingEmployee?.id],
    queryFn: () => fetchEmployeeEmployments(wsId, editingEmployee!.id),
    enabled: !!wsId && !!editingEmployee?.id && isOpen,
  });
  const activeEmployment = employments.find(e => e.status === "active") || employments[0];
  const activeLocationId = activeEmployment?.primary_location_id ?? "";
  const activeEmployerId = activeEmployment?.employer_id ?? "";

  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  const [selectedUserId, setSelectedUserId] = useState(editingEmployee?.user_id ?? "");
  const [selectedLocationId, setSelectedLocationId] = useState("");
  const [selectedEmployerId, setSelectedEmployerId] = useState("");

  useEffect(() => {
    setSelectedUserId(editingEmployee?.user_id ?? "");
    setSelectedLocationId(editingEmployee ? activeLocationId : "");
    setSelectedEmployerId(editingEmployee ? activeEmployerId : "");
  }, [editingEmployee, activeLocationId, activeEmployerId]);

  useEffect(() => {
    if (isOpen && wsId) {
      void fetchWorkspaceMembers(wsId).then(setMembers).catch(() => { /* empty */ });
    }
  }, [isOpen, wsId]);

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data: any = {
      full_name: formData.get("full_name") as string,
      employee_id_number: formData.get("employee_id_number") as string,
      hire_date: formData.get("hire_date") as string,
      is_active: formData.get("is_active") === "on",
    };
    if (selectedUserId) {
      data.user_id = selectedUserId;
    }

    if (editingEmployee) {
      try {
        await updateMutation.mutateAsync({ id: editingEmployee.id, employee: data });
        if (activeEmployment && (selectedLocationId !== activeLocationId || selectedEmployerId !== activeEmployerId)) {
          await updateEmployeeEmployment(wsId, editingEmployee.id, activeEmployment.id, {
            primary_location_id: selectedLocationId || null,
            employer_id: selectedEmployerId || undefined,
          });
        }
      } catch {
        // error handling inside mutation
      }
    } else {
      if (selectedLocationId) {
        data.primary_location_id = selectedLocationId;
      }
      data.employer_id = selectedEmployerId ? selectedEmployerId : null;
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
          editingEmployee={editingEmployee}
          employers={employers}
          locations={locations ?? []}
          members={members}
          prefillEnrollmentId={prefillEnrollmentId}
          selectedEmployerId={selectedEmployerId}
          selectedLocationId={selectedLocationId}
          selectedUserId={selectedUserId}
          setSelectedEmployerId={setSelectedEmployerId}
          setSelectedLocationId={setSelectedLocationId}
          setSelectedUserId={setSelectedUserId}
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

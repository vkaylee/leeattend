import { useQuery } from "@tanstack/react-query";
import { useBlocker, useNavigate, useSearch } from "@tanstack/react-router";
import { ArrowLeft, BriefcaseBusiness, UserCircle } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CreateEmployeeRequest } from "@/lib/api";

import EmployeeFormInputs from "../../components/features/employees/employee-form-inputs";
import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { useToast } from "@/components/ui/toast";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { createEmployee, fetchEmployers } from "@/lib/api";
import { createQueryKey, employeeKeys } from "@/lib/query-keys";
import { useLocations } from "@/pages/settings/hooks/use-locations";

function getFormString(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === "string" ? value : "";
}

export default function EmployeeCreatePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const search = useSearch({ strict: false });
  const [selectedLocationId, setSelectedLocationId] = useState("");
  const [selectedEmployerId, setSelectedEmployerId] = useState("");
  const [isDirty, setIsDirty] = useState(false);
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { data: employers } = useQuery({
    queryKey: createQueryKey("employers"),
    queryFn: fetchEmployers,
  });
  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    withResolver: true,
  });

  const createMutation = useOrganizationMutation<
    CreateEmployeeRequest,
    Awaited<ReturnType<typeof createEmployee>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: createEmployee,
    onSuccess: (employee) => {
      setIsDirty(false);
      toast.success(t("common.success"));
      void navigate({ to: "/employees/$employeeId", params: { employeeId: employee.id } });
    },
  });

  const handleCancel = () => {
    void navigate({ to: "/employees" });
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const payload: CreateEmployeeRequest = {
      full_name: getFormString(formData, "full_name"),
      employee_id_number: getFormString(formData, "employee_id_number"),
      hire_date: getFormString(formData, "hire_date"),
      employer_id: selectedEmployerId || undefined,
      primary_location_id: selectedLocationId || undefined,
      tags: [],
    };
    void createMutation.mutateAsync(payload);
  };

  return (
    <div className="mx-auto max-w-4xl space-y-6 pb-20 animate-in fade-in duration-500">
      <header className="flex flex-col gap-5 border-b border-slate-200/80 pb-6 dark:border-slate-800/80 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex items-center gap-3 sm:gap-4">
          <Button aria-label={t("common.back", { defaultValue: "Quay lại" })} className="shrink-0 p-3" onClick={handleCancel} size="icon" type="button" variant="ghost">
            <ArrowLeft size={18} />
          </Button>
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-200/70 bg-primary-50 text-primary-600 dark:border-primary-800/60 dark:bg-primary-950/50 dark:text-primary-400">
              <UserCircle aria-hidden="true" size={22} />
            </div>
            <div className="min-w-0">
              <h1 className="text-2xl font-bold tracking-tight text-slate-900 dark:text-slate-100" tabIndex={-1}>{t("employees.form.add_title")}</h1>
              <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t("employees.form.subtitle")}</p>
            </div>
          </div>
        </div>
        <div className="grid w-full grid-cols-2 gap-3 lg:w-auto">
          <Button onClick={handleCancel} size="lg" type="button" variant="outline">{t("employees.form.cancel").toUpperCase()}</Button>
          <Button data-testid="save-employee-btn" form="employee-create-form" loading={createMutation.isPending} size="lg" type="submit" variant="secondary">{t("employees.form.save").toUpperCase()}</Button>
        </div>
      </header>

      <form className="grid grid-cols-1 gap-6 lg:grid-cols-2" id="employee-create-form" onChange={() => { setIsDirty(true); }} onSubmit={handleSubmit}>
        <section aria-labelledby="employee-basic-heading" className="space-y-6 rounded-2xl border border-slate-100 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
          <div className="flex items-center gap-3 border-b border-slate-100 pb-4 dark:border-slate-800">
            <UserCircle aria-hidden="true" className="text-primary-600 dark:text-primary-400" size={20} />
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100" id="employee-basic-heading">{t("employees.form.basic_info", { defaultValue: "Thông tin cơ bản" })}</h2>
          </div>
          <EmployeeFormInputs
            activeEmployment={undefined}
            editingEmployee={null}
            employers={[]}
            locations={[]}
            members={[]}
            prefillEnrollmentId={search.enrollment_id ?? ""}
            selectedEmployerId=""
            selectedLocationId=""
            selectedUserId=""
            setSelectedEmployerId={() => undefined}
            setSelectedLocationId={() => undefined}
            setSelectedUserId={() => undefined}
            showUserLink={false}
            t={t}
            basicOnly
          />
        </section>
        <section aria-labelledby="employee-employment-heading" className="space-y-6 rounded-2xl border border-slate-100 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
          <div className="flex items-center gap-3 border-b border-slate-100 pb-4 dark:border-slate-800">
            <BriefcaseBusiness aria-hidden="true" className="text-primary-600 dark:text-primary-400" size={20} />
            <h2 className="text-base font-bold text-slate-900 dark:text-slate-100" id="employee-employment-heading">{t("employees.form.employment_info", { defaultValue: "Quan hệ lao động" })}</h2>
          </div>
          <EmployeeFormInputs
            activeEmployment={undefined}
            editingEmployee={null}
            employers={employers ?? []}
            locations={locations ?? []}
            locationsUnavailable={locationsUnavailable}
            members={[]}
            prefillEnrollmentId=""
            selectedEmployerId={selectedEmployerId}
            selectedLocationId={selectedLocationId}
            selectedUserId=""
            setSelectedEmployerId={setSelectedEmployerId}
            setSelectedLocationId={setSelectedLocationId}
            setSelectedUserId={() => undefined}
            showUserLink={false}
            t={t}
            employmentOnly
          />
        </section>
      </form>

      <ConfirmDialog
        cancelLabel={t("common.cancel", { defaultValue: "Ở lại" })}
        confirmLabel={t("common.confirm", { defaultValue: "Rời đi" })}
        isOpen={blocker.status === "blocked"}
        message={t("employees.form.unsaved_changes", { defaultValue: "Bạn có thay đổi chưa lưu. Rời trang sẽ mất dữ liệu." })}
        onClose={() => { blocker.reset?.(); }}
        onConfirm={() => { blocker.proceed?.(); }}
        title={t("common.warning", { defaultValue: "Cảnh báo" })}
        variant="warning"
      />
    </div>
  );
}

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useBlocker, useNavigate, useSearch } from "@tanstack/react-router";
import { ArrowLeft, BriefcaseBusiness, Info, UserCircle } from "lucide-react";
import React, { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { CreateEmployeeRequest, CreateEmployeeTenureRequest, EmployeeMatchCandidate } from "@/lib/api";

import EmployeeFormInputs from "@/components/features/employees/employee-form-inputs";
import EmployeeIdentityPreflightPanel from "@/components/features/employees/employee-identity-preflight-panel";
import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { useMyPermissions } from "@/hooks/use-permissions";
import { createEmployee, createEmployeeTenure, fetchEmployers, resolveStagingLog } from "@/lib/api";
import { ApiError } from "@/lib/api/errors";
import { createQueryKey, employeeKeys } from "@/lib/query-keys";
import { useEmployeeIdentityPreflight } from "@/pages/employees/hooks/use-employee-identity-preflight";
import { useLocations } from "@/pages/settings/hooks/use-locations";

interface EmployeeCreateSearch {
  enrollment_id?: string;
  staging_log_id?: string;
}

function normalizeOptionalField(raw: string): string | undefined {
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function useEmployeeCreateController(search: EmployeeCreateSearch) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { hasPermission } = useMyPermissions();
  const canMatchIdentity = hasPermission("employees:identity:match");
  const canRehire = hasPermission("employees:rehire");

  const [fullName, setFullName] = useState("");
  const [employeeCode, setEmployeeCode] = useState(search.enrollment_id ?? "");
  const [hireDate, setHireDate] = useState("");
  const [phone, setPhone] = useState("");
  const [dateOfBirth, setDateOfBirth] = useState("");
  const [nationalId, setNationalId] = useState("");
  const [selectedLocationId, setSelectedLocationId] = useState("");
  const [selectedEmployerId, setSelectedEmployerId] = useState("");
  const [employerError, setEmployerError] = useState<string | undefined>(undefined);
  const [isDirty, setIsDirty] = useState(false);

  const [selectedReuseCandidate, setSelectedReuseCandidate] = useState<EmployeeMatchCandidate | null>(null);
  const [softAcknowledged, setSoftAcknowledged] = useState(false);
  const [conflictMessage, setConflictMessage] = useState<string | null>(null);
  const [isResolvingStaging, setIsResolvingStaging] = useState(false);
  const submitGuardRef = useRef(false);

  const preflight = useEmployeeIdentityPreflight({ enabled: canMatchIdentity, fullName, dateOfBirth, phone, nationalId });

  const invalidateStaging = useCallback(async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs") }),
      queryClient.invalidateQueries({ queryKey: createQueryKey("staging-logs-count") }),
      queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-raw-logs") }),
    ]);
  }, [queryClient]);

  const finishWithEmployee = useCallback(async (employeeId: string) => {
    if (search.staging_log_id) {
      await resolveStagingLog(search.staging_log_id, employeeId);
      await invalidateStaging();
    }
    setIsDirty(false);
    toast.success(t("common.success"));
    void navigate({ to: "/employees/$employeeId", params: { employeeId } });
  }, [invalidateStaging, navigate, search.staging_log_id, t, toast]);

  const handleWriteError = useCallback((error: unknown) => {
    if (error instanceof ApiError && error.isConflict) {
      const message = t("employees.preflight.errors.conflict", {
        defaultValue: "Dữ liệu trùng lặp hoặc trạng thái hồ sơ đã thay đổi. Vui lòng kiểm tra lại kết quả khớp.",
      });
      setConflictMessage(message);
      toast.error(message);
      if (canMatchIdentity) preflight.retry();
      return;
    }
    toast.error(error instanceof Error ? error.message : t("common.error"));
  }, [canMatchIdentity, preflight, t, toast]);

  const createMutation = useOrganizationMutation<CreateEmployeeRequest, Awaited<ReturnType<typeof createEmployee>>>({
    keys: [employeeKeys.all()],
    mutationFn: createEmployee,
    onSuccess: (employee) => { void finishWithEmployee(employee.id); },
    onError: handleWriteError,
  });

  const reuseTenureMutation = useOrganizationMutation<
    { body: CreateEmployeeTenureRequest; employeeId: string },
    Awaited<ReturnType<typeof createEmployeeTenure>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: ({ employeeId, body }) => createEmployeeTenure(employeeId, body),
    onSuccess: (created) => { void finishWithEmployee(created.employee_id); },
    onError: handleWriteError,
  });

  const markDirty = useCallback(() => {
    setIsDirty(true);
    setConflictMessage(null);
  }, []);

  const handleIdentityFieldChange = useCallback(
    (setter: (val: string) => void) => (val: string) => {
      setter(val);
      setSoftAcknowledged(false);
      setSelectedReuseCandidate(null);
      markDirty();
    },
    [markDirty],
  );

  const handleAssignStagingCandidate = useCallback((candidate: EmployeeMatchCandidate) => {
    if (!search.staging_log_id || isResolvingStaging) return;
    setIsResolvingStaging(true);
    void resolveStagingLog(search.staging_log_id, candidate.employee_id)
      .then(async () => {
        await invalidateStaging();
        setIsDirty(false);
        toast.success(t("common.success"));
        void navigate({ to: "/attendance-logs", search: { tab: "unknown", mode: "this_month" } });
      })
      .catch((error: unknown) => { toast.error(error instanceof Error ? error.message : t("common.error")); })
      .finally(() => { setIsResolvingStaging(false); });
  }, [invalidateStaging, isResolvingStaging, navigate, search.staging_log_id, t, toast]);

  const isPending = createMutation.isPending || reuseTenureMutation.isPending || isResolvingStaging;
  const isBlockedForCreate =
    preflight.status === "hard" ||
    preflight.status === "loading" ||
    preflight.status === "stale" ||
    (preflight.status === "soft" && !softAcknowledged);
  const isSubmitDisabled = isPending || (!selectedReuseCandidate && isBlockedForCreate);

  const submitReuse = () => {
    if (!selectedReuseCandidate) return;
    if (!selectedEmployerId) {
      setEmployerError(t("employees.preflight.errors.employer_required_for_reuse", { defaultValue: "Vui lòng chọn pháp nhân trực thuộc để tạo giai đoạn mới." }));
      return;
    }
    submitGuardRef.current = true;
    void reuseTenureMutation
      .mutateAsync({
        employeeId: selectedReuseCandidate.employee_id,
        body: {
          start_date: hireDate,
          employer_id: selectedEmployerId,
          primary_location_id: selectedLocationId || undefined,
          expected_employee_code: selectedReuseCandidate.employee_code,
          expected_status: selectedReuseCandidate.status === "terminated" ? "terminated" : undefined,
        },
      })
      .catch(() => undefined)
      .finally(() => { submitGuardRef.current = false; });
  };

  const handleSubmit = (event: React.SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submitGuardRef.current || isSubmitDisabled) return;
    setConflictMessage(null);
    setEmployerError(undefined);
    if (selectedReuseCandidate) {
      submitReuse();
      return;
    }
    submitGuardRef.current = true;
    void createMutation
      .mutateAsync({
        full_name: fullName,
        employee_id_number: employeeCode,
        hire_date: hireDate,
        employer_id: selectedEmployerId || undefined,
        primary_location_id: selectedLocationId || undefined,
        tags: [],
        phone: canMatchIdentity ? normalizeOptionalField(phone) : undefined,
        date_of_birth: canMatchIdentity ? normalizeOptionalField(dateOfBirth) : undefined,
        national_id: canMatchIdentity ? normalizeOptionalField(nationalId) : undefined,
      })
      .catch(() => undefined)
      .finally(() => { submitGuardRef.current = false; });
  };

  return {
    canMatchIdentity, canRehire, conflictMessage, dateOfBirth, employeeCode, employerError, fullName,
    handleAssignStagingCandidate, handleIdentityFieldChange, handleSubmit, hireDate, isDirty, isPending,
    isResolvingStaging, isSubmitDisabled, markDirty, nationalId, phone, preflight, selectedEmployerId,
    selectedLocationId, selectedReuseCandidate, setConflictMessage, setDateOfBirth, setEmployeeCode,
    setEmployerError, setFullName, setHireDate, setNationalId, setPhone, setSelectedEmployerId,
    setSelectedLocationId, setSelectedReuseCandidate, setSoftAcknowledged, softAcknowledged, t,
    handleCancel: () => { void navigate({ to: "/employees" }); },
  };
}

export default function EmployeeCreatePage() {
  const rawSearch = useSearch({ strict: false });
  const search: EmployeeCreateSearch = {
    enrollment_id: typeof rawSearch.enrollment_id === "string" ? rawSearch.enrollment_id : undefined,
    staging_log_id: typeof rawSearch.staging_log_id === "string" ? rawSearch.staging_log_id : undefined,
  };
  const ctrl = useEmployeeCreateController(search);
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { data: employers } = useQuery({ queryKey: createQueryKey("employers"), queryFn: fetchEmployers });
  const blocker = useBlocker({ shouldBlockFn: () => ctrl.isDirty, withResolver: true });
  const hasStagingPrefill = Boolean(search.enrollment_id ?? search.staging_log_id);

  return (
    <div className="mx-auto max-w-6xl space-y-6 px-4 pb-20 animate-in fade-in duration-500 sm:px-6">
      <header className="flex flex-col gap-5 border-b border-slate-200/80 pb-6 dark:border-slate-800/80 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex items-center gap-3 sm:gap-4">
          <Button aria-label={ctrl.t("common.back", { defaultValue: "Quay lại" })} className="shrink-0 p-3" onClick={ctrl.handleCancel} size="icon" type="button" variant="ghost">
            <ArrowLeft size={18} />
          </Button>
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-200/70 bg-primary-50 text-primary-600 dark:border-primary-800/60 dark:bg-primary-950/50 dark:text-primary-400">
              <UserCircle aria-hidden="true" size={22} />
            </div>
            <div className="min-w-0">
              <Typography className="text-2xl font-bold tracking-tight text-slate-900 dark:text-slate-100" tabIndex={-1} variant="h1">{ctrl.t("employees.form.add_title")}</Typography>
              <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">{ctrl.t("employees.form.subtitle")}</Typography>
            </div>
          </div>
        </div>
        <div className="grid w-full grid-cols-2 gap-3 lg:w-auto">
          <Button onClick={ctrl.handleCancel} size="lg" type="button" variant="outline">{ctrl.t("employees.form.cancel").toUpperCase()}</Button>
          <Button data-testid="save-employee-btn" disabled={ctrl.isSubmitDisabled} form="employee-create-form" loading={ctrl.isPending} size="lg" type="submit" variant="secondary">
            {ctrl.selectedReuseCandidate
              ? ctrl.t("employees.preflight.confirm_reuse_submit", { defaultValue: "Tạo giai đoạn mới" }).toUpperCase()
              : ctrl.t("employees.form.save").toUpperCase()}
          </Button>
        </div>
      </header>

      {hasStagingPrefill ? (
        <div className="flex items-start gap-3 rounded-2xl border border-amber-200 bg-amber-50/70 p-4 text-sm text-amber-900 dark:border-amber-900/50 dark:bg-amber-950/30 dark:text-amber-200" data-testid="staging-prefill-notice" role="status">
          <Info aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
          <Typography className="text-sm text-inherit" variant="p">
            {ctrl.t("employees.preflight.staging_notice", {
              defaultValue: "Mã chấm công chỉ dùng để điền sẵn mã nhân viên, không phải bằng chứng định danh. Vui lòng nhập số điện thoại, ngày sinh hoặc CCCD/CMND để kiểm tra hồ sơ trùng trước khi tạo mới.",
            })}
          </Typography>
        </div>
      ) : null}

      <EmployeeIdentityPreflightPanel
        candidates={ctrl.preflight.candidates}
        canRehire={ctrl.canRehire}
        conflictMessage={ctrl.conflictMessage}
        errorKind={ctrl.preflight.errorKind}
        hasStagingLog={Boolean(search.staging_log_id)}
        isAssigningStaging={ctrl.isResolvingStaging}
        onAcknowledgeSoft={() => { ctrl.setSoftAcknowledged((prev) => !prev); }}
        onAssignStagingCandidate={ctrl.handleAssignStagingCandidate}
        onCancelReuse={() => { ctrl.setSelectedReuseCandidate(null); ctrl.setEmployerError(undefined); }}
        onRetry={ctrl.preflight.retry}
        onSelectReuse={(candidate) => { ctrl.setSelectedReuseCandidate(candidate); ctrl.setConflictMessage(null); ctrl.setEmployerError(undefined); }}
        selectedReuseCandidate={ctrl.selectedReuseCandidate}
        softAcknowledged={ctrl.softAcknowledged}
        status={ctrl.preflight.status}
        t={ctrl.t}
      />

      <form className="grid grid-cols-1 gap-6 lg:grid-cols-2" id="employee-create-form" onSubmit={ctrl.handleSubmit}>
        <section aria-labelledby="employee-basic-heading" className="space-y-6 rounded-2xl border border-slate-100 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
          <div className="flex items-center gap-3 border-b border-slate-100 pb-4 dark:border-slate-800">
            <UserCircle aria-hidden="true" className="text-primary-600 dark:text-primary-400" size={20} />
            <Typography className="text-base font-bold text-slate-900 dark:text-slate-100" id="employee-basic-heading" variant="h2">{ctrl.t("employees.form.basic_info", { defaultValue: "Thông tin cơ bản" })}</Typography>
          </div>
          <EmployeeFormInputs
            activeEmployment={undefined}
            basicOnly
            dateOfBirth={ctrl.dateOfBirth}
            editingEmployee={null}
            employeeCode={ctrl.employeeCode}
            employers={[]}
            fullName={ctrl.fullName}
            hireDate={ctrl.hireDate}
            locations={[]}
            nationalId={ctrl.nationalId}
            onDateOfBirthChange={ctrl.handleIdentityFieldChange(ctrl.setDateOfBirth)}
            onEmployeeCodeChange={(val) => { ctrl.setEmployeeCode(val); ctrl.markDirty(); }}
            onFullNameChange={ctrl.handleIdentityFieldChange(ctrl.setFullName)}
            onHireDateChange={(val) => { ctrl.setHireDate(val); ctrl.markDirty(); }}
            onNationalIdChange={ctrl.handleIdentityFieldChange(ctrl.setNationalId)}
            onPhoneChange={ctrl.handleIdentityFieldChange(ctrl.setPhone)}
            phone={ctrl.phone}
            prefillEnrollmentId={search.enrollment_id ?? ""}
            reuseMode={Boolean(ctrl.selectedReuseCandidate)}
            selectedEmployerId=""
            selectedLocationId=""
            setSelectedEmployerId={() => undefined}
            setSelectedLocationId={() => undefined}
            showIdentityFields={ctrl.canMatchIdentity}
            t={ctrl.t}
          />
        </section>
        <section aria-labelledby="employee-employment-heading" className="space-y-6 rounded-2xl border border-slate-100 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-8">
          <div className="flex items-center gap-3 border-b border-slate-100 pb-4 dark:border-slate-800">
            <BriefcaseBusiness aria-hidden="true" className="text-primary-600 dark:text-primary-400" size={20} />
            <Typography className="text-base font-bold text-slate-900 dark:text-slate-100" id="employee-employment-heading" variant="h2">{ctrl.t("employees.form.employment_info", { defaultValue: "Quan hệ lao động" })}</Typography>
          </div>
          <EmployeeFormInputs
            activeEmployment={undefined}
            editingEmployee={null}
            employerError={ctrl.employerError}
            employers={employers ?? []}
            employmentOnly
            locations={locations}
            locationsUnavailable={locationsUnavailable}
            prefillEnrollmentId=""
            selectedEmployerId={ctrl.selectedEmployerId}
            selectedLocationId={ctrl.selectedLocationId}
            setSelectedEmployerId={(val) => { ctrl.setSelectedEmployerId(val); ctrl.setEmployerError(undefined); ctrl.markDirty(); }}
            setSelectedLocationId={(val) => { ctrl.setSelectedLocationId(val); ctrl.markDirty(); }}
            t={ctrl.t}
          />
        </section>
      </form>

      <ConfirmDialog
        cancelLabel={ctrl.t("common.cancel", { defaultValue: "Ở lại" })}
        confirmLabel={ctrl.t("common.confirm", { defaultValue: "Rời đi" })}
        isOpen={blocker.status === "blocked"}
        message={ctrl.t("employees.form.unsaved_changes", { defaultValue: "Bạn có thay đổi chưa lưu. Rời trang sẽ mất dữ liệu." })}
        onClose={() => { blocker.reset?.(); }}
        onConfirm={() => { blocker.proceed?.(); }}
        title={ctrl.t("common.warning", { defaultValue: "Cảnh báo" })}
        variant="warning"
      />
    </div>
  );
}

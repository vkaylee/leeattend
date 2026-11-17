import type { TFunction } from "i18next";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ClipboardList, UserCheck } from "lucide-react";
import { useMemo, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";

import { z } from "zod";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useAllEmployees } from "@/hooks/use-employees";
import { lifecycleKeys, useLifecycleTemplates } from "@/hooks/use-lifecycle";
import { createLifecycleChecklist, type Employee, type LifecycleChecklistTemplate } from "@/lib/api";

const assignSchema = z.object({
  employment_id: z.string().min(1, "Vui lòng chọn nhân sự có hợp đồng lao động đang hoạt động"),
  template_id: z.string().min(1, "Vui lòng chọn mẫu quy trình checklist"),
});

export interface LifecycleAssignModalProps {
  employeeName?: string;
  employmentId?: string;
  isOpen: boolean;
  onClose: () => void;
}

type Translation = TFunction;
interface SelectOption {
  description?: string;
  label: string;
  value: string;
}

export const formatChecklistTypeLabel = (type: string, t: Translation): string => {
  const labels: Record<string, string> = {
    onboarding: "lifecycle.types.onboarding",
    offboarding: "lifecycle.types.offboarding",
    probation_end: "lifecycle.types.probation",
    transfer: "lifecycle.types.transfer",
  };
  const defaultValues: Record<string, string> = {
    onboarding: "Tiếp nhận nhân sự mới (Onboarding)",
    offboarding: "Bàn giao nghỉ việc (Offboarding)",
    probation_end: "Đánh giá hết thử việc (Probation)",
    transfer: "Điều chuyển vị trí / luân chuyển (Transfer)",
  };
  const normalizedType = type.toLowerCase();
  return labels[normalizedType]
    ? t(labels[normalizedType], { defaultValue: defaultValues[normalizedType] })
    : type.toUpperCase();
};

function getErrorMessage(error: unknown): string {
  return error instanceof Error && error.message ? error.message : "Không thể khởi tạo quy trình checklist";
}

function useLifecycleForm({
  employmentId: initialEmploymentId,
  employeeName: initialEmployeeName,
  isOpen,
  onClose,
}: LifecycleAssignModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [selectedEmployeeId, setSelectedEmployeeId] = useState("");
  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [previousOpen, setPreviousOpen] = useState(isOpen);
  const { employees } = useAllEmployees({ enabled: isOpen && !initialEmploymentId });
  const { data: templates = [], isLoading: isLoadingTemplates } = useLifecycleTemplates();

  if (isOpen !== previousOpen) {
    setPreviousOpen(isOpen);
    if (isOpen) {
      setSelectedTemplateId("");
      setErrorMsg(null);
      if (!initialEmploymentId) setSelectedEmployeeId("");
    }
  }

  const selectedEmployee = useMemo(() => {
    if (initialEmploymentId) return null;
    return employees.find((employee: Employee) => employee.id === selectedEmployeeId);
  }, [employees, initialEmploymentId, selectedEmployeeId]);
  const effectiveEmploymentId = initialEmploymentId || selectedEmployee?.active_employment_id || "";
  const effectiveEmployeeName = initialEmployeeName || selectedEmployee?.full_name || "";
  const activeTemplates = useMemo(
    () => templates.filter((template: LifecycleChecklistTemplate) => template.is_active),
    [templates],
  );
  const mutation = useMutation({
    mutationFn: async () => {
      const parsed = assignSchema.parse({ employment_id: effectiveEmploymentId, template_id: selectedTemplateId });
      return createLifecycleChecklist(parsed.employment_id, { template_id: parsed.template_id });
    },
    onError: (error: unknown) => { setErrorMsg(getErrorMessage(error)); },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
      toast.success(t("lifecycle.assign_success", { defaultValue: `Đã khởi tạo quy trình thành công cho ${effectiveEmployeeName}` }));
      onClose();
    },
  });

  const submit = (event: FormEvent): void => {
    event.preventDefault();
    if (!effectiveEmploymentId) {
      setErrorMsg(t("lifecycle.errors.no_active_employment", { defaultValue: "Nhân sự được chọn không có hồ sơ việc làm (employment) đang hoạt động." }));
      return;
    }
    if (!selectedTemplateId) {
      setErrorMsg(t("lifecycle.errors.no_template_selected", { defaultValue: "Vui lòng chọn một mẫu quy trình." }));
      return;
    }
    setErrorMsg(null);
    mutation.mutate();
  };

  return {
    activeTemplates,
    effectiveEmployeeName,
    effectiveEmploymentId,
    employees,
    errorMsg,
    isLoadingTemplates,
    isPending: mutation.isPending,
    selectedEmployee,
    selectedEmployeeId,
    selectedTemplateId,
    setSelectedEmployeeId,
    setSelectedTemplateId,
    submit,
    t,
  };
}

function EmployeePicker({
  employees,
  selectedEmployee,
  selectedEmployeeId,
  setSelectedEmployeeId,
  t,
}: {
  employees: Employee[];
  selectedEmployee: Employee | undefined;
  selectedEmployeeId: string;
  setSelectedEmployeeId: (value: string) => void;
  t: Translation;
}) {
  const options: SelectOption[] = employees
    .filter((employee) => Boolean(employee.active_employment_id))
    .map((employee) => ({
      value: employee.id,
      label: `${employee.full_name}${employee.employee_id_number ? ` (${employee.employee_id_number})` : ""}`,
      description: employee.department_name ?? employee.job_title ?? undefined,
    }));

  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="lifecycle-employee-combobox">
        {t("lifecycle.fields.employee", { defaultValue: "Nhân sự áp dụng" })} *
      </label>
      <Combobox
        clearOnEscape
        data-testid="lifecycle-employee-combobox"
        id="lifecycle-employee-combobox"
        onChange={setSelectedEmployeeId}
        options={options}
        placeholder={t("lifecycle.placeholders.select_employee", { defaultValue: "Tìm kiếm nhân sự đang công tác..." })}
        value={selectedEmployeeId}
      />
      {!!selectedEmployee && !selectedEmployee.active_employment_id && (
        <p className="text-xs text-rose-500 font-medium">
          {t("lifecycle.errors.employee_has_no_employment", { defaultValue: "Nhân sự này hiện không có hồ sơ công tác hợp lệ." })}
        </p>
      )}
    </div>
  );
}

function LockedEmployee({ employeeName, t }: { employeeName: string; t: Translation }) {
  return (
    <div className="p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 flex items-center gap-3">
      <div className="w-9 h-9 rounded-xl bg-primary-100 dark:bg-primary-900/40 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold">
        <UserCheck size={18} />
      </div>
      <div>
        <Typography className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase" variant="p">{employeeName}</Typography>
        <Typography className="text-[11px] text-slate-400 font-mono" variant="p">
          {t("lifecycle.labels.active_employment", { defaultValue: "Hồ sơ công tác hiện hành" })}
        </Typography>
      </div>
    </div>
  );
}

function TemplatePicker({
  activeTemplates,
  isLoading,
  selectedTemplateId,
  setSelectedTemplateId,
  t,
}: {
  activeTemplates: LifecycleChecklistTemplate[];
  isLoading: boolean;
  selectedTemplateId: string;
  setSelectedTemplateId: (value: string) => void;
  t: Translation;
}) {
  const options: SelectOption[] = activeTemplates.map((template) => ({
    value: template.id,
    label: template.name,
    description: formatChecklistTypeLabel(template.checklist_type, t),
  }));

  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="lifecycle-template-combobox">
        {t("lifecycle.fields.template", { defaultValue: "Mẫu quy trình Checklist" })} *
      </label>
      <Combobox
        clearOnEscape
        data-testid="lifecycle-template-combobox"
        disabled={isLoading}
        id="lifecycle-template-combobox"
        onChange={setSelectedTemplateId}
        options={options}
        placeholder={t("lifecycle.placeholders.select_template", { defaultValue: "Chọn mẫu quy trình (Onboarding, Thử việc, Bàn giao...)" })}
        value={selectedTemplateId}
      />
    </div>
  );
}

function LifecycleFooter({
  canSubmit,
  isPending,
  onClose,
  t,
}: { canSubmit: boolean; isPending: boolean; onClose: () => void; t: Translation }) {
  return (
    <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
      <Button disabled={isPending} onClick={onClose} type="button" variant="ghost">{t("common.cancel", { defaultValue: "HỦY" })}</Button>
      <Button disabled={isPending || !canSubmit} type="submit" variant="secondary">{t("lifecycle.actions.submit_assign", { defaultValue: "KHỞI TẠO QUY TRÌNH" })}</Button>
    </div>
  );
}

export default function LifecycleAssignModal(props: LifecycleAssignModalProps) {
  const { employeeName: initialEmployeeName, employmentId: initialEmploymentId, isOpen, onClose } = props;
  const form = useLifecycleForm(props);
  const { t } = form;

  return (
    <Modal
      icon={<ClipboardList size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={initialEmploymentId
        ? t("lifecycle.modal.subtitle_employee", { defaultValue: `Gán checklist công việc cho nhân sự ${form.effectiveEmployeeName}` })
        : t("lifecycle.modal.subtitle_hub", { defaultValue: "Khởi tạo quy trình mới (Onboarding, Thử việc, Bàn giao) cho nhân sự" })}
      title={t("lifecycle.modal.title", { defaultValue: "Khởi tạo Quy trình Vòng đời" })}
    >
      <form className="space-y-6 pt-4" onSubmit={form.submit}>
        <div className="space-y-4">
          {!initialEmploymentId && (
            <EmployeePicker
              employees={form.employees}
              selectedEmployee={form.selectedEmployee ?? undefined}
              selectedEmployeeId={form.selectedEmployeeId}
              setSelectedEmployeeId={form.setSelectedEmployeeId}
              t={t}
            />
          )}
          {!!initialEmploymentId && <LockedEmployee employeeName={initialEmployeeName || form.effectiveEmployeeName} t={t} />}
          <TemplatePicker
            activeTemplates={form.activeTemplates}
            isLoading={form.isLoadingTemplates}
            selectedTemplateId={form.selectedTemplateId}
            setSelectedTemplateId={form.setSelectedTemplateId}
            t={t}
          />
          {!!form.errorMsg && (
            <div className="p-3 text-xs font-semibold text-rose-600 bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 rounded-xl" role="alert">
              {form.errorMsg}
            </div>
          )}
        </div>
        <LifecycleFooter
          canSubmit={Boolean(form.effectiveEmploymentId && form.selectedTemplateId)}
          isPending={form.isPending}
          onClose={onClose}
          t={t}
        />
      </form>
    </Modal>
  );
}

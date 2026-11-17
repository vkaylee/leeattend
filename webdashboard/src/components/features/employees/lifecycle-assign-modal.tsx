import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ClipboardList, UserCheck } from "lucide-react";
import { useMemo, useState, type SyntheticEvent } from "react";
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

type Translate = (key: string, options?: { defaultValue?: string }) => string;

interface ComboboxOption {
  description?: string;
  label: string;
  value: string;
}

export interface LifecycleAssignModalProps {
  employeeName?: string;
  employmentId?: string;
  isOpen: boolean;
  onClose: () => void;
}

interface EmployeeSelectorProps {
  employees: Employee[];
  onChange: (value: string) => void;
  selectedEmployeeId: string;
  t: Translate;
}

interface LockedEmployeeInfoProps {
  employeeName: string;
  t: Translate;
}

interface TemplateSelectorProps {
  isLoading: boolean;
  onChange: (value: string) => void;
  selectedTemplateId: string;
  t: Translate;
  templates: LifecycleChecklistTemplate[];
}

interface AssignmentFieldsProps {
  effectiveEmployeeName: string;
  employees: Employee[];
  errorMsg: string | null;
  initialEmploymentId?: string;
  isLoadingTemplates: boolean;
  onEmployeeChange: (value: string) => void;
  onTemplateChange: (value: string) => void;
  selectedEmployeeId: string;
  selectedTemplateId: string;
  t: Translate;
  templates: LifecycleChecklistTemplate[];
}

interface AssignmentFormActionsProps {
  canSubmit: boolean;
  isPending: boolean;
  onClose: () => void;
  t: Translate;
}

export function formatChecklistTypeLabel(type: string, t: Translate): string {
  switch (type.toLowerCase()) {
    case "onboarding":
      return t("lifecycle.types.onboarding", { defaultValue: "Tiếp nhận nhân sự mới (Onboarding)" });
    case "offboarding":
      return t("lifecycle.types.offboarding", { defaultValue: "Bàn giao nghỉ việc (Offboarding)" });
    case "probation_end":
      return t("lifecycle.types.probation", { defaultValue: "Đánh giá hết thử việc (Probation)" });
    case "transfer":
      return t("lifecycle.types.transfer", { defaultValue: "Điều chuyển vị trí / luân chuyển (Transfer)" });
    default:
      return type.toUpperCase();
  }
}

function EmployeeSelector({ employees, onChange, selectedEmployeeId, t }: EmployeeSelectorProps) {
  const options: ComboboxOption[] = employees
    .filter((employee) => Boolean(employee.active_employment_id))
    .map((employee) => ({
      description: employee.department_name ?? employee.job_title ?? undefined,
      label: `${employee.full_name}${employee.employee_id_number ? ` (${employee.employee_id_number})` : ""}`,
      value: employee.id,
    }));

  return (
    <div className="space-y-2">
      <label
        className="text-[11px] font-bold text-slate-500 uppercase tracking-widest"
        htmlFor="lifecycle-employee-combobox"
      >
        {t("lifecycle.fields.employee", { defaultValue: "Nhân sự áp dụng" })} *
      </label>
      <Combobox
        clearOnEscape
        data-testid="lifecycle-employee-combobox"
        id="lifecycle-employee-combobox"
        onChange={onChange}
        options={options}
        placeholder={t("lifecycle.placeholders.select_employee", {
          defaultValue: "Tìm kiếm nhân sự đang công tác...",
        })}
        value={selectedEmployeeId}
      />
    </div>
  );
}

function LockedEmployeeInfo({ employeeName, t }: LockedEmployeeInfoProps) {
  return (
    <div className="p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 flex items-center gap-3">
      <div className="w-9 h-9 rounded-xl bg-primary-100 dark:bg-primary-900/40 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold">
        <UserCheck size={18} />
      </div>
      <div>
        <Typography className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase" variant="p">
          {employeeName}
        </Typography>
        <Typography className="text-[11px] text-slate-400 font-mono" variant="p">
          {t("lifecycle.labels.active_employment", { defaultValue: "Hồ sơ công tác hiện hành" })}
        </Typography>
      </div>
    </div>
  );
}

function TemplateSelector({ isLoading, onChange, selectedTemplateId, templates, t }: TemplateSelectorProps) {
  const options: ComboboxOption[] = templates.map((template) => ({
    description: formatChecklistTypeLabel(template.checklist_type, t),
    label: template.name,
    value: template.id,
  }));

  return (
    <div className="space-y-2">
      <label
        className="text-[11px] font-bold text-slate-500 uppercase tracking-widest"
        htmlFor="lifecycle-template-combobox"
      >
        {t("lifecycle.fields.template", { defaultValue: "Mẫu quy trình Checklist" })} *
      </label>
      <Combobox
        clearOnEscape
        data-testid="lifecycle-template-combobox"
        disabled={isLoading}
        id="lifecycle-template-combobox"
        onChange={onChange}
        options={options}
        placeholder={t("lifecycle.placeholders.select_template", {
          defaultValue: "Chọn mẫu quy trình (Onboarding, Thử việc, Bàn giao...)",
        })}
        value={selectedTemplateId}
      />
    </div>
  );
}

function AssignmentFields({
  effectiveEmployeeName,
  errorMsg,
  initialEmploymentId,
  employees,
  isLoadingTemplates,
  onEmployeeChange,
  onTemplateChange,
  selectedEmployeeId,
  selectedTemplateId,
  templates,
  t,
}: AssignmentFieldsProps) {
  return (
    <div className="space-y-4">
      {initialEmploymentId ? (
        <LockedEmployeeInfo employeeName={effectiveEmployeeName} t={t} />
      ) : (
        <EmployeeSelector
          employees={employees}
          onChange={onEmployeeChange}
          selectedEmployeeId={selectedEmployeeId}
          t={t}
        />
      )}
      <TemplateSelector
        isLoading={isLoadingTemplates}
        onChange={onTemplateChange}
        selectedTemplateId={selectedTemplateId}
        t={t}
        templates={templates}
      />
      {errorMsg ? (
        <div className="p-3 text-xs font-semibold text-rose-600 bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 rounded-xl">
          {errorMsg}
        </div>
      ) : null}
    </div>
  );
}

function AssignmentFormActions({ canSubmit, isPending, onClose, t }: AssignmentFormActionsProps) {
  return (
    <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
      <Button disabled={isPending} onClick={onClose} type="button" variant="ghost">
        {t("common.cancel", { defaultValue: "HỦY" })}
      </Button>
      <Button disabled={isPending || !canSubmit} type="submit" variant="secondary">
        {t("lifecycle.actions.submit_assign", { defaultValue: "KHỞI TẠO QUY TRÌNH" })}
      </Button>
    </div>
  );
}

interface AssignmentFormState {
  errorMsg: string | null;
  selectedEmployeeId: string;
  selectedTemplateId: string;
}

function useAssignmentFormState(isOpen: boolean, initialEmploymentId?: string): AssignmentFormState & {
  setErrorMsg: (value: string | null) => void;
  setSelectedEmployeeId: (value: string) => void;
  setSelectedTemplateId: (value: string) => void;
} {
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>("");
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [prevIsOpen, setPrevIsOpen] = useState(isOpen);

  if (isOpen !== prevIsOpen) {
    setPrevIsOpen(isOpen);
    if (isOpen) {
      setSelectedTemplateId("");
      setErrorMsg(null);
      if (!initialEmploymentId) {
        setSelectedEmployeeId("");
      }
    }
  }

  return {
    errorMsg,
    selectedEmployeeId,
    selectedTemplateId,
    setErrorMsg,
    setSelectedEmployeeId,
    setSelectedTemplateId,
  };
}

function useSelectedEmployee(
  employees: Employee[],
  initialEmploymentId: string | undefined,
  selectedEmployeeId: string,
): Employee | undefined {
  return useMemo(() => {
    if (initialEmploymentId) return undefined;
    return employees.find((employee) => employee.id === selectedEmployeeId);
  }, [employees, initialEmploymentId, selectedEmployeeId]);
}

function getEffectiveEmploymentId(initialEmploymentId: string | undefined, activeEmploymentId: string | null | undefined): string {
  return initialEmploymentId?.length ? initialEmploymentId : activeEmploymentId ?? "";
}

function getEffectiveEmployeeName(initialEmployeeName: string | undefined, selectedEmployeeName: string | undefined): string {
  return initialEmployeeName?.length ? initialEmployeeName : selectedEmployeeName ?? "";
}

function getValidationError(
  employmentId: string,
  templateId: string,
  t: Translate,
): string | null {
  if (!employmentId) {
    return t("lifecycle.errors.no_active_employment", {
      defaultValue: "Nhân sự được chọn không có hồ sơ việc làm (employment) đang hoạt động.",
    });
  }
  if (!templateId) {
    return t("lifecycle.errors.no_template_selected", {
      defaultValue: "Vui lòng chọn một mẫu quy trình.",
    });
  }
  return null;
}

function getMutationErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  return "Không thể khởi tạo quy trình checklist";
}

export default function LifecycleAssignModal({
  employeeName: initialEmployeeName,
  employmentId: initialEmploymentId,
  isOpen,
  onClose,
}: LifecycleAssignModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const {
    errorMsg,
    selectedEmployeeId,
    selectedTemplateId,
    setErrorMsg,
    setSelectedEmployeeId,
    setSelectedTemplateId,
  } = useAssignmentFormState(isOpen, initialEmploymentId);
  const { employees } = useAllEmployees({ enabled: isOpen && !initialEmploymentId });
  const { data: templates = [], isLoading: isLoadingTemplates } = useLifecycleTemplates();
  const activeTemplates = useMemo(
    () => templates.filter((template) => template.is_active),
    [templates],
  );
  const selectedEmployee = useSelectedEmployee(employees, initialEmploymentId, selectedEmployeeId);
  const effectiveEmploymentId = getEffectiveEmploymentId(initialEmploymentId, selectedEmployee?.active_employment_id);
  const effectiveEmployeeName = getEffectiveEmployeeName(initialEmployeeName, selectedEmployee?.full_name);

  const mutation = useMutation({
    mutationFn: () => {
      const parsed = assignSchema.parse({
        employment_id: effectiveEmploymentId,
        template_id: selectedTemplateId,
      });
      return createLifecycleChecklist(parsed.employment_id, {
        template_id: parsed.template_id,
      });
    },
    onError: (error: unknown) => {
      setErrorMsg(getMutationErrorMessage(error));
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
      toast.success(
        t("lifecycle.assign_success", {
          defaultValue: `Đã khởi tạo quy trình thành công cho ${effectiveEmployeeName}`,
        }),
      );
      onClose();
    },
  });

  const handleSubmit = (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    const validationError = getValidationError(effectiveEmploymentId, selectedTemplateId, t);
    if (validationError) {
      setErrorMsg(validationError);
      return;
    }
    setErrorMsg(null);
    mutation.mutate();
  };

  const subtitle = initialEmploymentId
    ? t("lifecycle.modal.subtitle_employee", {
        defaultValue: `Gán checklist công việc cho nhân sự ${effectiveEmployeeName}`,
      })
    : t("lifecycle.modal.subtitle_hub", {
        defaultValue: "Khởi tạo quy trình mới (Onboarding, Thử việc, Bàn giao) cho nhân sự",
      });

  return (
    <Modal
      icon={<ClipboardList size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={subtitle}
      title={t("lifecycle.modal.title", { defaultValue: "Khởi tạo Quy trình Vòng đời" })}
    >
      <form className="space-y-6 pt-4" onSubmit={handleSubmit}>
        <AssignmentFields
          effectiveEmployeeName={effectiveEmployeeName}
          employees={employees}
          errorMsg={errorMsg}
          initialEmploymentId={initialEmploymentId}
          isLoadingTemplates={isLoadingTemplates}
          onEmployeeChange={setSelectedEmployeeId}
          onTemplateChange={setSelectedTemplateId}
          selectedEmployeeId={selectedEmployeeId}
          selectedTemplateId={selectedTemplateId}
          t={t}
          templates={activeTemplates}
        />
        <AssignmentFormActions
          canSubmit={Boolean(effectiveEmploymentId && selectedTemplateId)}
          isPending={mutation.isPending}
          onClose={onClose}
          t={t}
        />
      </form>
    </Modal>
  );
}

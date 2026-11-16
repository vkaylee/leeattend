import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ClipboardList, UserCheck } from "lucide-react";
import { useMemo, useState, type SubmitEvent } from "react";
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

export function formatChecklistTypeLabel(type: string, t: (k: string, opts?: { defaultValue?: string }) => string): string {
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

export default function LifecycleAssignModal({
  isOpen,
  onClose,employmentId: initialEmploymentId,
  employeeName: initialEmployeeName,
}: LifecycleAssignModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

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

  const { employees } = useAllEmployees({ enabled: isOpen && !initialEmploymentId });
  const { data: templates = [], isLoading: isLoadingTemplates } = useLifecycleTemplates();

  // Active templates only
  const activeTemplates = useMemo(() => {
    return templates.filter((tpl: LifecycleChecklistTemplate) => tpl.is_active);
  }, [templates]);

  // Derive target employment ID and employee name
  const selectedEmployee = useMemo(() => {
    if (initialEmploymentId) return null;
    return employees.find((emp: Employee) => emp.id === selectedEmployeeId);
  }, [employees, selectedEmployeeId, initialEmploymentId]);

  const effectiveEmploymentId = initialEmploymentId === undefined || initialEmploymentId === ""
    ? selectedEmployee?.active_employment_id ?? ""
    : initialEmploymentId;
  const effectiveEmployeeName = initialEmployeeName === undefined || initialEmployeeName === ""
    ? selectedEmployee?.full_name ?? ""
    : initialEmployeeName;

  const mutation = useMutation({
    mutationFn: async () => {
      const parsed = assignSchema.parse({
        employment_id: effectiveEmploymentId,
        template_id: selectedTemplateId,
      });
      return createLifecycleChecklist(parsed.employment_id, {
        template_id: parsed.template_id,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
      toast.success(
        t("lifecycle.assign_success", {
          defaultValue: `Đã khởi tạo quy trình thành công cho ${effectiveEmployeeName}`,
        })
      );
      onClose();
    },
    onError: (err: unknown) => {
      const error = err as Error;
      setErrorMsg(error.message || "Không thể khởi tạo quy trình checklist");
    },
  });

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    if (!effectiveEmploymentId) {
      setErrorMsg(
        t("lifecycle.errors.no_active_employment", {
          defaultValue: "Nhân sự được chọn không có hồ sơ việc làm (employment) đang hoạt động.",
        })
      );
      return;
    }
    if (!selectedTemplateId) {
      setErrorMsg(
        t("lifecycle.errors.no_template_selected", {
          defaultValue: "Vui lòng chọn một mẫu quy trình.",
        })
      );
      return;
    }
    setErrorMsg(null);
    mutation.mutate();
  };

  const templateOptions = activeTemplates.map((tpl: LifecycleChecklistTemplate) => ({
    value: tpl.id,
    label: tpl.name,
    description: formatChecklistTypeLabel(tpl.checklist_type, t),
  }));

  const employeeOptions = employees
    .filter((emp: Employee) => Boolean(emp.active_employment_id))
    .map((emp: Employee) => ({
      value: emp.id,
      label: `${emp.full_name}${emp.employee_id_number ? ` (${emp.employee_id_number})` : ""}`,
      description: emp.department_name ?? emp.job_title ?? undefined,
    }));

  return (
    <Modal
      icon={<ClipboardList size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={
        initialEmploymentId
          ? t("lifecycle.modal.subtitle_employee", {
              defaultValue: `Gán checklist công việc cho nhân sự ${effectiveEmployeeName}`,
            })
          : t("lifecycle.modal.subtitle_hub", {
              defaultValue: "Khởi tạo quy trình mới (Onboarding, Thử việc, Bàn giao) cho nhân sự",
            })
      }
      title={t("lifecycle.modal.title", { defaultValue: "Khởi tạo Quy trình Vòng đời" })}
    >
      <form className="space-y-6 pt-4" onSubmit={handleSubmit}>
        <div className="space-y-4">
          {/* Employee Selection (if in Hub mode) */}
          {!initialEmploymentId && (
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
                onChange={setSelectedEmployeeId}
                options={employeeOptions}
                placeholder={t("lifecycle.placeholders.select_employee", {
                  defaultValue: "Tìm kiếm nhân sự đang công tác...",
                })}
                value={selectedEmployeeId}
              />
              {!!selectedEmployee && !selectedEmployee.active_employment_id && (
                <Typography className="text-xs text-rose-500 font-medium" variant="p">
                  {t("lifecycle.errors.employee_has_no_employment", {
                    defaultValue: "Nhân sự này hiện không có hồ sơ công tác hợp lệ.",
                  })}
                </Typography>
              )}
            </div>
          )}

          {/* Locked Employee info (if in Employee Detail mode) */}
          {Boolean(initialEmploymentId) && (
            <div className="p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-primary-100 dark:bg-primary-900/40 text-primary-600 dark:text-primary-400 flex items-center justify-center font-bold">
                <UserCheck size={18} />
              </div>
              <div>
                <Typography className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase" variant="p">
                  {effectiveEmployeeName}
                </Typography>
                <Typography className="text-[11px] text-slate-400 font-mono" variant="p">
                  {t("lifecycle.labels.active_employment", { defaultValue: "Hồ sơ công tác hiện hành" })}
                </Typography>
              </div>
            </div>
          )}

          {/* Template Selection */}
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
              disabled={isLoadingTemplates}
              id="lifecycle-template-combobox"
              onChange={setSelectedTemplateId}
              options={templateOptions}
              placeholder={t("lifecycle.placeholders.select_template", {
                defaultValue: "Chọn mẫu quy trình (Onboarding, Thử việc, Bàn giao...)",
              })}
              value={selectedTemplateId}
            />
          </div>

          {!!errorMsg && (
            <div className="p-3 text-xs font-semibold text-rose-600 bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900 rounded-xl">
              {errorMsg}
            </div>
          )}
        </div>

        <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button disabled={mutation.isPending} onClick={onClose} type="button" variant="ghost">
            {t("common.cancel", { defaultValue: "HỦY" })}
          </Button>
          <Button
            disabled={mutation.isPending || !effectiveEmploymentId || !selectedTemplateId}
            type="submit"
            variant="secondary"
          >
            {t("lifecycle.actions.submit_assign", { defaultValue: "KHỞI TẠO QUY TRÌNH" })}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

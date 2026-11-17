import type { TFunction } from "i18next";

import { Briefcase } from "lucide-react";
import { useEffect, useState } from "react";
import { Controller, useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useCreateEmployeeAssignment, useUpdateEmployeeAssignment } from "@/hooks/use-employee-assignments";
import { useAllEmployees } from "@/hooks/use-employees";
import { type EmployeeAssignment } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";

const assignmentSchema = z.object({
  host_location_id: z.string().optional(),
  start_date: z.string().min(1, { message: "Ngày bắt đầu là bắt buộc" }),
  end_date: z.string().optional(),
  reason: z.string().optional(),
  status: z.enum(["scheduled", "active", "completed", "cancelled"]).optional(),
}).superRefine((data, ctx) => {
  if (data.start_date && data.end_date && new Date(data.start_date) > new Date(data.end_date)) {
    ctx.addIssue({ code: "custom", message: "Ngày kết thúc phải sau ngày bắt đầu", path: ["end_date"] });
  }
});

type AssignmentFormValues = z.infer<typeof assignmentSchema>;
type EditableAssignment = EmployeeAssignment & { employee_id?: string; employee_name?: string; employee_code?: string };
type Translation = TFunction;

export interface EmployeeAssignmentFormModalProps {
  editingAssignment: EditableAssignment | null;
  employeeId?: string;
  isOpen: boolean;
  onClose: () => void;
}

function getDefaultFormValues(): AssignmentFormValues {
  return { status: "scheduled", start_date: new Date().toISOString().split("T")[0], end_date: "", host_location_id: "", reason: "" };
}

function getErrorMessage(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

function useAssignmentForm({ editingAssignment, employeeId, isOpen, onClose }: EmployeeAssignmentFormModalProps) {
  const toast = useToast();
  const createAssignment = useCreateEmployeeAssignment();
  const updateAssignment = useUpdateEmployeeAssignment();
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { employees } = useAllEmployees({ enabled: isOpen && !employeeId });
  const targetInitialEmployeeId = editingAssignment?.employee_id ?? employeeId ?? "";
  const [selectedEmployeeId, setSelectedEmployeeId] = useState(targetInitialEmployeeId);
  const [previousState, setPreviousState] = useState({ editingAssignment, isOpen });
  const { control, formState: { errors, isSubmitting }, handleSubmit, register, reset } = useForm<AssignmentFormValues>({
    resolver: standardSchemaResolver(assignmentSchema), defaultValues: getDefaultFormValues(),
  });

  if (isOpen !== previousState.isOpen || editingAssignment !== previousState.editingAssignment) {
    setPreviousState({ editingAssignment, isOpen });
    if (isOpen) setSelectedEmployeeId(targetInitialEmployeeId);
  }

  useEffect(() => {
    if (!isOpen) return;
    reset(editingAssignment ? {
      status: editingAssignment.status,
      start_date: editingAssignment.start_date,
      end_date: editingAssignment.end_date ?? "",
      host_location_id: editingAssignment.host_location_id,
      reason: editingAssignment.reason ?? "",
    } : getDefaultFormValues());
  }, [editingAssignment, isOpen, reset]);

  const onSubmit = async (data: AssignmentFormValues): Promise<void> => {
    try {
      const targetEmployeeId = employeeId || editingAssignment?.employee_id || selectedEmployeeId;
      if (!targetEmployeeId) { toast.error("Vui lòng chọn nhân sự"); return; }
      const payloadDate = data.end_date ?? null;
      const payloadLocationId = data.host_location_id ?? null;
      if (editingAssignment) {
        await updateAssignment.mutateAsync({ employeeId: targetEmployeeId, assignmentId: editingAssignment.id, data: {
          end_date: payloadDate, reason: data.reason, status: data.status === "scheduled" ? undefined : data.status,
        } });
        toast.success("Đã cập nhật phái cử thành công");
      } else {
        if (!payloadLocationId) throw new Error("Vui lòng chọn địa điểm phái cử");
        await createAssignment.mutateAsync({ employeeId: targetEmployeeId, data: {
          start_date: data.start_date, end_date: payloadDate, host_location_id: payloadLocationId, reason: data.reason,
        } });
        toast.success("Đã tạo phái cử thành công");
      }
      onClose();
    } catch (error: unknown) { toast.error(getErrorMessage(error, "Có lỗi xảy ra")); }
  };

  return { control, employees, errors, handleSubmit, isSubmitting, locations, locationsUnavailable, onSubmit, register, selectedEmployeeId, setSelectedEmployeeId };
}

type AssignmentFormState = ReturnType<typeof useAssignmentForm>;

function EmployeeSelector({ editingAssignment, employees, employeeId, selectedEmployeeId, setSelectedEmployeeId, t }: {
  editingAssignment: EditableAssignment | null;
  employees: AssignmentFormState["employees"];
  employeeId?: string;
  selectedEmployeeId: string;
  setSelectedEmployeeId: (employeeId: string) => void;
  t: Translation;
}) {
  if (employeeId) return null;
  return <div className="space-y-2">
    <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="assignment-employee-combobox">{t("assignments.fields.employee", "Nhân sự")} *</label>
    {editingAssignment ? <div className="px-3 py-2.5 bg-slate-100 dark:bg-slate-800 rounded-xl text-sm font-semibold text-slate-800 dark:text-slate-200">
      {editingAssignment.employee_name || "Nhân sự"} {editingAssignment.employee_code ? `(${editingAssignment.employee_code})` : ""}
    </div> : <Combobox clearOnEscape data-testid="assignment-employee-combobox" id="assignment-employee-combobox" onChange={setSelectedEmployeeId} options={employees.map((employee) => ({
      value: employee.id, label: `${employee.full_name} (${employee.employee_id_number})`, description: employee.department_name ?? employee.job_title ?? undefined,
    }))} placeholder={t("assignments.placeholders.select_employee", "Chọn nhân sự phái cử...")} value={selectedEmployeeId} />}
  </div>;
}

function AssignmentDates({ errors, register, t }: Pick<AssignmentFormState, "errors" | "register"> & { t: Translation }) {
  return <div className="grid grid-cols-2 gap-4">
    <FormField error={errors.start_date?.message} label={t("employees.assignments.start_date", "Ngày bắt đầu")} required type="date" {...register("start_date")} />
    <FormField error={errors.end_date?.message} label={t("employees.assignments.end_date", "Ngày kết thúc")} type="date" {...register("end_date")} />
  </div>;
}

function AssignmentLocation({ control, errors, locations, locationsUnavailable, t }: Pick<AssignmentFormState, "control" | "errors" | "locations" | "locationsUnavailable"> & { t: Translation }) {
  return <div aria-labelledby="assignment-location-label" className="space-y-2" role="group">
    <span className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" id="assignment-location-label">Địa điểm đến (Tùy chọn)</span>
    {locationsUnavailable ? <p className="text-sm text-slate-500 dark:text-slate-400" id="assignment-location-unavailable" role="status">{t("employees.form.location_unavailable")}</p> : <Controller control={control} name="host_location_id" render={({ field }) => <Combobox
      clearOnEscape data-testid="assignment-location-combobox" id="assignment-location-combobox" onChange={field.onChange}
      options={locations.map((location) => ({ value: location.id, label: location.name, description: location.is_headquarters ? "Headquarters" : location.timezone }))}
      placeholder="Chọn địa điểm làm việc mới" value={field.value ?? ""}
    />} />}
    {!!errors.host_location_id?.message && <Typography className="text-sm font-semibold text-rose-500 mt-1" variant="p">{errors.host_location_id.message}</Typography>}
  </div>;
}

function AssignmentStatus({ register }: Pick<AssignmentFormState, "register">) {
  return <div className="space-y-2"><label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="status">Trạng thái
    <select className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm" id="status" {...register("status")}>
      <option value="scheduled">Đã lên lịch</option><option value="active">Đang hiệu lực</option><option value="completed">Đã hoàn thành</option><option value="cancelled">Đã hủy bỏ</option>
    </select>
  </label></div>;
}

function AssignmentFooter({ isEditing, isSubmitting, onClose, t }: { isEditing: boolean; isSubmitting: boolean; onClose: () => void; t: Translation }) {
  return <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
    <Button disabled={isSubmitting} onClick={onClose} type="button" variant="outline">{t("common.cancel", "HỦY BỎ")}</Button>
    <Button loading={isSubmitting} type="submit" variant="secondary">{isEditing ? "CẬP NHẬT" : "TẠO MỚI"}</Button>
  </div>;
}

export default function EmployeeAssignmentFormModal({ editingAssignment, employeeId, isOpen, onClose }: EmployeeAssignmentFormModalProps) {
  const { t } = useTranslation();
  const form = useAssignmentForm({ editingAssignment, employeeId, isOpen, onClose });
  return <Modal icon={<Briefcase size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose}
    subtitle={editingAssignment ? "Cập nhật thông tin điều chuyển/phái cử nhân sự" : "Tạo lệnh phái cử mới cho nhân viên đến địa điểm khác"}
    title={editingAssignment ? "Chỉnh sửa Phái cử" : "Tạo Phái cử Mới"}>
    <form className="space-y-6 pt-4" onSubmit={form.handleSubmit(form.onSubmit)}><div className="space-y-4">
      <EmployeeSelector editingAssignment={editingAssignment} employeeId={employeeId} employees={form.employees} selectedEmployeeId={form.selectedEmployeeId} setSelectedEmployeeId={form.setSelectedEmployeeId} t={t} />
      <AssignmentDates errors={form.errors} register={form.register} t={t} />
      <AssignmentLocation control={form.control} errors={form.errors} locations={form.locations} locationsUnavailable={form.locationsUnavailable} t={t} />
      {!!editingAssignment && <AssignmentStatus register={form.register} />}
      <FormField error={form.errors.reason?.message} label="Lý do / Ghi chú" placeholder="Ghi chú thêm về việc phái cử..." {...form.register("reason")} />
    </div><AssignmentFooter isEditing={Boolean(editingAssignment)} isSubmitting={form.isSubmitting} onClose={onClose} t={t} /></form>
  </Modal>;
}

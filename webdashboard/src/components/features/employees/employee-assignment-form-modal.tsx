import { Briefcase } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useForm, Controller } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  useCreateEmployeeAssignment,
  useUpdateEmployeeAssignment,
} from "@/hooks/use-employee-assignments";
import { useAllEmployees } from "@/hooks/use-employees";
import { type EmployeeAssignment } from "@/lib/api";
import { useLocations } from "@/pages/settings/hooks/use-locations";
const assignmentSchema = z
  .object({
    host_location_id: z.string().optional(),
    start_date: z.string().min(1, { message: "Ngày bắt đầu là bắt buộc" }),
    end_date: z.string().optional(),
    reason: z.string().optional(),
    status: z.enum(["scheduled", "active", "completed", "cancelled"]).optional(),
  })
  .superRefine((data, ctx) => {
    if (data.start_date && data.end_date) {
      if (new Date(data.start_date) > new Date(data.end_date)) {
        ctx.addIssue({
          code: "custom",
          message: "Ngày kết thúc phải sau ngày bắt đầu",
          path: ["end_date"],
        });
      }
    }
  });

type AssignmentFormValues = z.infer<typeof assignmentSchema>;

export interface EmployeeAssignmentFormModalProps {
  editingAssignment: (EmployeeAssignment & { employee_id?: string; employee_name?: string; employee_code?: string }) | null;
  employeeId?: string;
  isOpen: boolean;
  onClose: () => void;
}

function useAssignmentForm({
  isOpen,employeeId,
  editingAssignment,
  onClose,
}: {
  isOpen: boolean;
  employeeId?: string;
  editingAssignment: (EmployeeAssignment & { employee_id?: string; employee_name?: string; employee_code?: string }) | null;
  onClose: () => void;
}) {
  const toast = useToast();
  const createAssignment = useCreateEmployeeAssignment();
  const updateAssignment = useUpdateEmployeeAssignment();
  const { data: locations, isError: locationsUnavailable } = useLocations();
  const { employees } = useAllEmployees({ enabled: isOpen && !employeeId });
  const targetInitialEmployeeId = editingAssignment?.employee_id ?? employeeId ?? "";
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>(targetInitialEmployeeId);
  const [prevOpenState, setPrevOpenState] = useState({ editingAssignment, isOpen });

  if (isOpen !== prevOpenState.isOpen || editingAssignment !== prevOpenState.editingAssignment) {
    setPrevOpenState({ editingAssignment, isOpen });
    if (isOpen) {
      setSelectedEmployeeId(targetInitialEmployeeId);
    }
  }
  const {
    register,
    handleSubmit,
    reset,
    control,
    formState: { errors, isSubmitting },
  } = useForm<AssignmentFormValues>({
    resolver: standardSchemaResolver(assignmentSchema),
    defaultValues: {
      status: "scheduled",
      start_date: new Date().toISOString().split("T")[0],
      end_date: "",
      host_location_id: "",
      reason: "",
    },
  });

  useEffect(() => {
    if (isOpen) {
      if (editingAssignment) {
        reset({
          status: editingAssignment.status,
          start_date: editingAssignment.start_date,
          end_date: editingAssignment.end_date ?? "",
          host_location_id: editingAssignment.host_location_id,
          reason: editingAssignment.reason ?? "",
        });
      } else {
        reset({
          status: "scheduled",
          start_date: new Date().toISOString().split("T")[0],
          end_date: "",
          host_location_id: "",
          reason: "",
        });
      }
    }
  }, [isOpen, editingAssignment, reset]);

  const onSubmit = async (data: AssignmentFormValues) => {
    try {
      const targetEmployeeId = employeeId || editingAssignment?.employee_id || selectedEmployeeId;
      if (!targetEmployeeId) {
        toast.error("Vui lòng chọn nhân sự");
        return;
      }

      const payloadDate = data.end_date ?? null;
      const payloadLocationId = data.host_location_id ?? null;

      if (editingAssignment) {
        await updateAssignment.mutateAsync({employeeId: targetEmployeeId,
          assignmentId: editingAssignment.id,
          data: {
            end_date: payloadDate,
            reason: data.reason,
            status: data.status === "scheduled" ? undefined : data.status,
          },
        });
        toast.success("Đã cập nhật phái cử thành công");
        onClose();
      } else {
        if (!payloadLocationId) {
          throw new Error("Vui lòng chọn địa điểm phái cử");
        }
        await createAssignment.mutateAsync({employeeId: targetEmployeeId,
          data: {
            start_date: data.start_date,
            end_date: payloadDate,
            host_location_id: payloadLocationId,
            reason: data.reason,
          },
        });
        toast.success("Đã tạo phái cử thành công");
        onClose();
      }
    } catch (err: unknown) {
      const error = err as Error;
      toast.error(error.message || "Có lỗi xảy ra");
    }
  };

  return {
    register,
    handleSubmit,
    control,
    errors,
    isSubmitting,
    locations,
    locationsUnavailable,
    employees,
    selectedEmployeeId,
    setSelectedEmployeeId,
    onSubmit,
  };
}

export default function EmployeeAssignmentFormModal({
  isOpen,
  onClose,
  editingAssignment,employeeId,
}: EmployeeAssignmentFormModalProps) {
  const { t } = useTranslation();
  const {
    register,
    handleSubmit,
    control,
    errors,
    isSubmitting,
    locations,
    locationsUnavailable,
    employees,
    selectedEmployeeId,
    setSelectedEmployeeId,
    onSubmit,
  } = useAssignmentForm({
    isOpen,employeeId,
    editingAssignment,
    onClose,
  });

  return (
    <Modal
      icon={<Briefcase size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={
        editingAssignment
          ? "Cập nhật thông tin điều chuyển/phái cử nhân sự"
          : "Tạo lệnh phái cử mới cho nhân viên đến địa điểm khác"
      }
      title={editingAssignment ? "Chỉnh sửa Phái cử" : "Tạo Phái cử Mới"}
    >
      <form className="space-y-6 pt-4" onSubmit={handleSubmit(onSubmit)}>
        <div className="space-y-4">
          {!employeeId && (
            <div className="space-y-2">
              <label
                className="text-[11px] font-bold text-slate-500 uppercase tracking-widest"
                htmlFor="assignment-employee-combobox"
              >
                {t("assignments.fields.employee", "Nhân sự")} *
              </label>
              {editingAssignment ? (
                <div className="px-3 py-2.5 bg-slate-100 dark:bg-slate-800 rounded-xl text-sm font-semibold text-slate-800 dark:text-slate-200">
                  {editingAssignment.employee_name || "Nhân sự"} {editingAssignment.employee_code ? `(${editingAssignment.employee_code})` : ""}
                </div>
              ) : (
                <Combobox
                  clearOnEscape
                  data-testid="assignment-employee-combobox"
                  id="assignment-employee-combobox"
                  onChange={setSelectedEmployeeId}
                  options={employees.map((emp) => ({
                    value: emp.id,
                    label: `${emp.full_name} (${emp.employee_id_number})`,
                    description: emp.department_name ?? emp.job_title ?? undefined,
                  }))}
                  placeholder={t("assignments.placeholders.select_employee", "Chọn nhân sự phái cử...")}
                  value={selectedEmployeeId}
                />
              )}
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <FormField
                error={errors.start_date?.message}
                label={t("employees.assignments.start_date", "Ngày bắt đầu")}
                required
                type="date"
                {...register("start_date")}
              />
            </div>

            <FormField
              error={errors.end_date?.message}
              label={t("employees.assignments.end_date", "Ngày kết thúc")}
              type="date"
              {...register("end_date")}
            />
          </div>

          <div aria-labelledby="assignment-location-label" className="space-y-2" role="group">
            <span className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" id="assignment-location-label">
              Địa điểm đến (Tùy chọn)
            </span>
            {locationsUnavailable ? (
              <p className="text-sm text-slate-500 dark:text-slate-400" id="assignment-location-unavailable" role="status">
                {t("employees.form.location_unavailable")}
              </p>
            ) : (
              <Controller
                control={control}
                name="host_location_id"
                render={({ field }) => (
                  <Combobox
                    clearOnEscape
                    data-testid="assignment-location-combobox"
                    id="assignment-location-combobox"
                    onChange={field.onChange}
                    options={locations.map((loc) => ({
                      value: loc.id,
                      label: loc.name,
                      description: loc.is_headquarters ? "Headquarters" : loc.timezone,
                    }))}
                    placeholder="Chọn địa điểm làm việc mới"
                    value={field.value ?? ""}
                  />
                )}
              />
            )}
            {!!errors.host_location_id?.message && (
              <Typography className="text-sm font-semibold text-rose-500 mt-1" variant="p">
                {errors.host_location_id.message}
              </Typography>
            )}
          </div>

          {!!editingAssignment && (
            <div className="space-y-2">
              <label
                className="text-[11px] font-bold text-slate-500 uppercase tracking-widest"
                htmlFor="status"
              >
                Trạng thái
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm"
                  id="status"
                  {...register("status")}
                >
                  <option value="scheduled">Đã lên lịch</option>
                  <option value="active">Đang hiệu lực</option>
                  <option value="completed">Đã hoàn thành</option>
                  <option value="cancelled">Đã hủy bỏ</option>
                </select>
              </label>
            </div>
          )}

          <FormField
            error={errors.reason?.message}
            label="Lý do / Ghi chú"
            placeholder="Ghi chú thêm về việc phái cử..."
            {...register("reason")}
          />
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button disabled={isSubmitting} onClick={onClose} type="button" variant="outline">
            HỦY BỎ
          </Button>
          <Button loading={isSubmitting} type="submit" variant="secondary">
            {editingAssignment ? "CẬP NHẬT" : "TẠO MỚI"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

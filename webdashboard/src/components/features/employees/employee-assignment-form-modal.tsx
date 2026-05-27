import { Briefcase } from "lucide-react";
import React, { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import type { EmployeeAssignment } from "@/lib/api";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import {
  useCreateEmployeeAssignment,
  useUpdateEmployeeAssignment,
} from "@/hooks/use-employee-assignments";

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

interface EmployeeAssignmentFormModalProps {
  editingAssignment: EmployeeAssignment | null;
  employeeId: string;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function EmployeeAssignmentFormModal({
  isOpen,
  onClose,
  editingAssignment,
  wsId,
  employeeId,
}: EmployeeAssignmentFormModalProps) {
  const toast = useToast();
  const createAssignment = useCreateEmployeeAssignment();
  const updateAssignment = useUpdateEmployeeAssignment();

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

  const startDateValue = useWatch({ control, name: "start_date" });

  useEffect(() => {
    if (isOpen) {
      if (editingAssignment) {
        reset({
          status: editingAssignment.status,
          start_date: editingAssignment.start_date,
          end_date: editingAssignment.end_date ?? "",
          host_location_id: editingAssignment.host_location_id ?? "",
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
      const payloadDate = data.end_date ? data.end_date : null;
      const payloadLocationId = data.host_location_id ? data.host_location_id : null;

      if (editingAssignment) {
        await updateAssignment.mutateAsync({
          wsId,
          employeeId,
          assignmentId: editingAssignment.id,
          data: {
            ...data,
            end_date: payloadDate,
            host_location_id: payloadLocationId,
            status: data.status as any,
          },
        });
        toast.success("Đã cập nhật phái cử thành công");
        onClose();
      } else {
        await createAssignment.mutateAsync({
          wsId,
          employeeId,
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
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <FormField
                error={errors.start_date?.message}
                label="Ngày bắt đầu"
                required
                type="date"
                {...register("start_date")}
              />
            </div>

            <FormField
              error={errors.end_date?.message}
              label="Ngày kết thúc"
              type="date"
              {...register("end_date")}
            />
          </div>

          <div className="space-y-2">
            <FormField
              error={errors.host_location_id?.message}
              label="Mã UUID Địa điểm đến (Tùy chọn)"
              placeholder="VD: 123e4567-e89b-12d3-a456-426614174000"
              {...register("host_location_id")}
            />
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

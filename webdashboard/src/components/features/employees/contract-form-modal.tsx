import { Typography } from "@/components/ui/typography";
/* eslint-disable complexity */
/* eslint-disable perfectionist/sort-imports */
import { useQuery } from "@tanstack/react-query";
import { FileText } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useForm, useWatch } from "react-hook-form";

import { addMonths, addYears, format, subDays } from "date-fns";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import type { CreateContractRequest, EmployeeContract } from "@/lib/api";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { cn } from "@/lib/utils";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { fetchEmploymentPhases, fetchEmployee, fetchEmployeeEmployments } from "@/lib/api/employees";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

const contractSchema = z.object({
  contract_number: z.string().optional(),
  contract_type: z.enum(["probation", "training", "fixed_term", "indefinite"]),
  assign_phase_id: z.string().optional(),
  status: z.enum(["draft", "active", "expired", "terminated"]).optional(),
  start_date: z.string().min(1, { message: "Ngày hiệu lực là bắt buộc" }),
  end_date: z.string().optional(),
  notes: z.string().optional(),
}).superRefine((data, ctx) => {
  if (data.contract_type !== "indefinite") {
    if (!data.end_date || data.end_date.trim() === "") {
      ctx.addIssue({
        code: "custom",
        message: "Ngày hết hạn là bắt buộc đối với loại hợp đồng này",
        path: ["end_date"],
      });
    }
  }
  if (data.start_date && data.end_date) {
    if (new Date(data.start_date) > new Date(data.end_date)) {
      ctx.addIssue({
        code: "custom",
        message: "Ngày hết hạn phải sau ngày hiệu lực",
        path: ["end_date"],
      });
    }
  }
});

type ContractFormValues = z.infer<typeof contractSchema>;

interface ContractFormModalProps {
  editingContract: EmployeeContract | null;
  employeeId: string;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

// eslint-disable-next-line max-lines-per-function
export default function ContractFormModal({
  isOpen,
  onClose,
  editingContract,
  wsId,
  employeeId,
}: ContractFormModalProps) {
  const toast = useToast();
  const { createContract, updateContract } = useEmployeeContracts(wsId, employeeId);
  const { data: phases } = useQuery({ queryKey: ["employment-phases", wsId], queryFn: () => fetchEmploymentPhases(wsId), enabled: !!wsId });
  const { data: employee } = useQuery({ queryKey: ["employee", wsId, employeeId], queryFn: () => fetchEmployee(wsId, employeeId), enabled: !!wsId && !!employeeId && isOpen });
  const { data: employments = [] } = useQuery({
    queryKey: ["employee-employments", wsId, employeeId],
    queryFn: () => fetchEmployeeEmployments(wsId, employeeId),
    enabled: !!wsId && !!employeeId && isOpen,
  });
  const activeEmployment = employments.find(e => e.status === "active") || employments[0];
  const [overlapData, setOverlapData] = useState<CreateContractRequest | null>(null);
  const [isConfirmingOverlap, setIsConfirmingOverlap] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    control,
    setValue,
    formState: { errors, isSubmitting },
  } = useForm<ContractFormValues>({
    resolver: standardSchemaResolver(contractSchema),
    defaultValues: {
      contract_type: "probation",
      status: "active",
      start_date: new Date().toISOString().split("T")[0],
      end_date: "",
      contract_number: "",
      notes: "",
      assign_phase_id: "",
    },
  });

  const contractType = useWatch({
    control,
    name: "contract_type",
  });
  const isIndefinite = contractType === "indefinite";

  const startDateValue = useWatch({ control, name: "start_date" });

  const applyDuration = (months?: number, years?: number) => {
    if (!startDateValue) return;
    const start = new Date(startDateValue);
    if (isNaN(start.getTime())) return;

    let end = start;
    if (months) end = addMonths(end, months);
    if (years) end = addYears(end, years);
    
    end = subDays(end, 1);
    
    setValue("end_date", format(end, "yyyy-MM-dd"), { shouldValidate: true });
    
    if (contractType === "indefinite") {
      setValue("contract_type", "fixed_term", { shouldValidate: true });
    }
  };

  useEffect(() => {
    if (isOpen) {
      if (editingContract) {
        reset({
          contract_type: editingContract.contract_type,
          status: editingContract.status,
          start_date: editingContract.start_date,
          end_date: editingContract.end_date ?? "",
          contract_number: editingContract.contract_number ?? "",
          notes: editingContract.notes ?? "",
        });
      } else {
        reset({
          contract_type: "probation",
          status: "active",
          start_date: employee?.hire_date ?? new Date().toISOString().split("T")[0],
          end_date: "",
          contract_number: "",
          notes: "",
          assign_phase_id: "",
        });
      }
    }
  }, [isOpen, editingContract, reset, employee?.hire_date]);

  const onSubmit = async (data: ContractFormValues) => {
    try {
      // Clear end_date if indefinite
      const payloadDate = isIndefinite ? null : (data.end_date ?? null);
      
      if (editingContract) {
        await updateContract.mutateAsync({
          contractId: editingContract.id,
          data: {
            ...data,
            end_date: payloadDate,
          },
        });
        toast.success("Đã cập nhật hợp đồng thành công");
        onClose();
      } else {
        if (!data.assign_phase_id) {
          toast.error("Vui lòng chọn Cột mốc khởi đầu"); return;
        }

        const payload: CreateContractRequest = {
          ...data,
          employment_id: activeEmployment?.id ?? "",
          end_date: payloadDate,
          assign_phase_id: data.assign_phase_id ?? undefined,
        };

        try {
          await createContract.mutateAsync(payload);
          toast.success("Đã tạo hợp đồng thành công");
          onClose();
        } catch (err: unknown) {
          const error = err as Error;
          if (error.message.includes("overlapping_contract_exists")) {
            setOverlapData(payload);
            setIsConfirmingOverlap(true);
          } else {
            toast.error(error.message || "Có lỗi xảy ra");
          }
        }
      }
    } catch {
      toast.error("Có lỗi xảy ra");
    }
  };

  const handleConfirmOverlap = async () => {
    if (!overlapData) return;
    try {
      await createContract.mutateAsync({
        ...overlapData,
        auto_close_overlapping: true,
      });
      toast.success("Đã tạo hợp đồng thành công và tự động kết thúc hợp đồng cũ");
      setIsConfirmingOverlap(false);
      setOverlapData(null);
      onClose();
    } catch (err: unknown) {
      const error = err as Error;
      toast.error(error.message || "Có lỗi xảy ra");
    }
  };

  return (
    <>
      <Modal
        icon={<FileText size={24} />}
        isOpen={isOpen ? !isConfirmingOverlap : false}
        maxWidth="md"
        onClose={onClose}
        subtitle={editingContract ? "Cập nhật thông tin hợp đồng nhân sự" : "Tạo hợp đồng mới cho nhân viên"}
        title={editingContract ? "Chỉnh sửa Hợp đồng" : "Tạo Hợp đồng Mới"}
      >
      <form className="space-y-6 pt-4" onSubmit={handleSubmit(onSubmit)}>
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <FormField
              label="Số hợp đồng"
              {...register("contract_number")}
              error={errors.contract_number?.message}
              placeholder="VD: HD-001"
            />
            
            <div className="space-y-2">
              <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="contract_type">
                Loại hợp đồng
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm"
                  id="contract_type"
                  {...register("contract_type")}
                >
                  <option value="probation">Thử việc</option>
                  <option value="training">Đào tạo</option>
                  <option value="fixed_term">Xác định thời hạn</option>
                  <option value="indefinite">Không xác định thời hạn</option>
                </select>
              </label>
              {!!errors.contract_type && <Typography className="text-xs text-rose-500 font-medium" variant="p">{errors.contract_type.message}</Typography>}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <FormField
                label="Ngày hiệu lực"
                type="date"
                {...register("start_date")}
                error={errors.start_date?.message}
                required
              />
              <Typography className="text-[10px] text-slate-500 italic px-1" variant="p">
                Lưu ý: Ngày bắt đầu hợp đồng cũ nhất sẽ tự động cập nhật lại thành Ngày vào làm của nhân viên.
              </Typography>
            </div>
            
            <FormField
              disabled={isIndefinite}
              label="Ngày hết hạn"
              type="date"
              {...register("end_date")}
              error={errors.end_date?.message}
              required={!isIndefinite}
            />
          </div>

          <div className="flex flex-wrap items-center gap-2 pt-1">
            <span className="text-[11px] font-bold text-slate-500 uppercase tracking-widest mr-1">Tính nhanh:</span>
            {[
              { label: "1 Tháng", months: 1 },
              { label: "2 Tháng", months: 2 },
              { label: "3 Tháng", months: 3 },
              { label: "6 Tháng", months: 6 },
              { label: "1 Năm", years: 1 },
              { label: "2 Năm", years: 2 },
              { label: "3 Năm", years: 3 },
            ].map((option) => (
              <Button
                className="px-2.5 py-1.5 text-[11px] font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-300 h-auto shadow-none border-none normal-case tracking-normal hover:text-slate-800"
                key={option.label}
                onClick={() => { applyDuration(option.months, option.years); }}
                type="button"
                variant="ghost"
              >
                {option.label}
              </Button>
            ))}
            <Button
              className={cn(
                "px-2.5 py-1.5 text-[11px] font-medium rounded-lg transition-colors h-auto shadow-none border-none normal-case tracking-normal hover:text-slate-800",
                isIndefinite 
                  ? "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-400 ring-1 ring-primary-500/50 hover:bg-primary-100 hover:text-primary-700" 
                  : "bg-slate-100 hover:bg-slate-200 text-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-300"
              )}
              onClick={() => {
                setValue("contract_type", "indefinite", { shouldValidate: true });
                setValue("end_date", "", { shouldValidate: true });
              }}
              type="button"
              variant="ghost"
            >
              Lâu dài
            </Button>
          </div>

          {!editingContract && (
            <div className="space-y-2">
              <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="assign_phase_id">
                Chọn Cột Mốc Khởi Đầu
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm"
                  id="assign_phase_id"
                  {...register("assign_phase_id")}
                  required
                >
                  <option value="">-- Bắt buộc chọn Cột mốc --</option>
                  {phases?.filter(p => p.is_active).map(phase => (
                    <option key={phase.id} value={phase.id}>{phase.name} ({phase.phase_type})</option>
                  ))}
                </select>
              </label>
              {!!errors.assign_phase_id && <Typography className="text-xs text-rose-500 font-medium" variant="p">{errors.assign_phase_id.message}</Typography>}
            </div>
          )}

          {!!editingContract && (
            <div className="space-y-2">
              <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="status">
                Trạng thái
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm"
                  id="status"
                  {...register("status")}
                >
                  <option value="draft">Bản nháp</option>
                  <option value="active">Đang hiệu lực</option>
                  <option value="expired">Đã hết hạn</option>
                  <option value="terminated">Đã chấm dứt</option>
                </select>
              </label>
            </div>
          )}

          <FormField
            label="Ghi chú"
            {...register("notes")}
            error={errors.notes?.message}
            placeholder="Ghi chú thêm về hợp đồng..."
          />
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button disabled={isSubmitting} onClick={onClose} type="button" variant="outline">
            HỦY BỎ
          </Button>
          <Button loading={isSubmitting} type="submit" variant="secondary">
            {editingContract ? "CẬP NHẬT" : "TẠO MỚI"}
          </Button>
        </div>
      </form>
    </Modal>

      <Modal
        icon={<FileText size={24} />}
        isOpen={isConfirmingOverlap}
        onClose={() => { setIsConfirmingOverlap(false); }}
        subtitle="Hệ thống phát hiện có sự trùng lặp thời gian"
        title="Trùng Lặp Hợp Đồng"
      >
        <div className="space-y-6 pt-4">
          <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">
            Ngày bắt đầu của hợp đồng mới đang giao nhau với một hợp đồng hiện tại đang còn hiệu lực của nhân viên.
          </Typography>
          <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">
            Bạn có muốn hệ thống tự động kết thúc hợp đồng cũ vào ngày trước đó để nhường chỗ cho hợp đồng mới này không?
          </Typography>
          <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
            <Button onClick={() => { setIsConfirmingOverlap(false); }} type="button" variant="outline">
              QUAY LẠI SỬA NGÀY
            </Button>
            <Button loading={createContract.isPending} onClick={() => { void handleConfirmOverlap(); }} type="button" variant="secondary">
              TỰ ĐỘNG ĐÓNG & TẠO MỚI
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
}

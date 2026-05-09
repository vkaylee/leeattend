import { FileText } from "lucide-react";
import React, { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import type { EmployeeContract } from "@/lib/api";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

const contractSchema = z.object({
  contract_number: z.string().optional(),
  contract_type: z.enum(["probation", "training", "fixed_term", "indefinite"]),
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

  const {
    register,
    handleSubmit,
    reset,
    control,
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
    },
  });

  const contractType = useWatch({
    control,
    name: "contract_type",
  });
  const isIndefinite = contractType === "indefinite";

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
          start_date: new Date().toISOString().split("T")[0],
          end_date: "",
          contract_number: "",
          notes: "",
        });
      }
    }
  }, [isOpen, editingContract, reset]);

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
      } else {
        await createContract.mutateAsync({
          ...data,
          employee_id: employeeId,
          end_date: payloadDate,
        });
        toast.success("Đã tạo hợp đồng thành công");
      }
      onClose();
    } catch {
      toast.error("Có lỗi xảy ra khi lưu hợp đồng");
    }
  };

  return (
    <Modal
      icon={<FileText size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle="Vui lòng điền các thông tin bên dưới"
      title={editingContract ? "Cập nhật hợp đồng" : "Tạo hợp đồng mới"}
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
              {!!errors.contract_type && <p className="text-xs text-rose-500 font-medium">{errors.contract_type.message}</p>}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <FormField
              label="Ngày hiệu lực"
              type="date"
              {...register("start_date")}
              error={errors.start_date?.message}
              required
            />
            
            <FormField
              disabled={isIndefinite}
              label="Ngày hết hạn"
              type="date"
              {...register("end_date")}
              error={errors.end_date?.message}
              required={!isIndefinite}
            />
          </div>

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
  );
}

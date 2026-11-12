import { useQueries } from "@tanstack/react-query";
import { FileText } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useForm, useWatch, type UseFormRegister, type FieldErrors } from "react-hook-form";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { addMonths, addYears, format, subDays } from "date-fns";

import type { CreateContractRequest, EmployeeContract, Employment } from "@/lib/api";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchEmploymentPhases, fetchEmployeeEmployments } from "@/lib/api/employees";
import { logError } from "@/lib/logger";
import { cn } from "@/lib/utils";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

import { contractSchema, type ContractFormValues } from "./contract-form-schema";
// ── Types ───────────────────────────────────────────────────────────────────

interface ContractFormModalProps {
  editingContract: EmployeeContract | null;
  employeeId: string;
  isOpen: boolean;
  onClose: () => void;
}

interface ContractFormBodyProps {
  applyDuration: (months?: number, years?: number) => void;
  editingContract: EmployeeContract | null;
  errors: FieldErrors<ContractFormValues>;
  isIndefinite: boolean;
  isSubmitting: boolean;
  onClose: () => void;
  phases: { id: string; is_active: boolean; name: string; phase_type: string }[] | undefined;
  register: UseFormRegister<ContractFormValues>;
  setIndefinite: () => void;
}

interface OverlapModalProps {
  isOpen: boolean;
  isPending: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

// ── Constants ───────────────────────────────────────────────────────────────

const DURATION_PRESETS: { label: string; months?: number; years?: number }[] = [
  { label: "1 Tháng", months: 1 },
  { label: "2 Tháng", months: 2 },
  { label: "3 Tháng", months: 3 },
  { label: "6 Tháng", months: 6 },
  { label: "1 Năm", years: 1 },
  { label: "2 Năm", years: 2 },
  { label: "3 Năm", years: 3 },
];

// ── Sub-components ──────────────────────────────────────────────────────────

function OverlapConfirmationModal({ isPending, isOpen, onClose, onConfirm }: OverlapModalProps) {
  return (
    <Modal icon={<FileText size={24} />} isOpen={isOpen} onClose={onClose} subtitle="Hệ thống phát hiện có sự trùng lặp thời gian" title="Trùng Lặp Hợp Đồng">
      <div className="space-y-6 pt-4">
        <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">
          Ngày bắt đầu của hợp đồng mới đang giao nhau với một hợp đồng hiện tại đang còn hiệu lực của nhân viên.
        </Typography>
        <Typography className="text-sm text-slate-600 dark:text-slate-300" variant="p">
          Bạn có muốn hệ thống tự động kết thúc hợp đồng cũ vào ngày trước đó để nhường chỗ cho hợp đồng mới này không?
        </Typography>
        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} type="button" variant="outline">QUAY LẠI SỬA NGÀY</Button>
          <Button loading={isPending} onClick={onConfirm} type="button" variant="secondary">TỰ ĐỘNG ĐÓNG & TẠO MỚI</Button>
        </div>
      </div>
    </Modal>
  );
}

function PhaseSelector({ errors, phases, register }: {
  errors: FieldErrors<ContractFormValues>;
  phases: { id: string; is_active: boolean; name: string; phase_type: string }[] | undefined;
  register: UseFormRegister<ContractFormValues>;
}) {
  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="assign_phase_id">
        Chọn Cột Mốc Khởi Đầu
        <select className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm" id="assign_phase_id" {...register("assign_phase_id")} required>
          <option value="">-- Bắt buộc chọn Cột mốc --</option>
          {phases?.filter((p) => p.is_active).map((phase) => <option key={phase.id} value={phase.id}>{phase.name} ({phase.phase_type})</option>)}
        </select>
      </label>
      {errors.assign_phase_id ? <Typography className="text-xs text-rose-500 font-medium" variant="p">{errors.assign_phase_id.message}</Typography> : null}
    </div>
  );
}

function StatusSelector({ register }: { register: UseFormRegister<ContractFormValues> }) {
  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="status">Trạng thái
        <select className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm" id="status" {...register("status")}>
          <option value="draft">Bản nháp</option>
          <option value="active">Đang hiệu lực</option>
          <option value="expired">Đã hết hạn</option>
          <option value="terminated">Đã chấm dứt</option>
        </select>
      </label>
    </div>
  );
}

// eslint-disable-next-line complexity -- form rendering naturally has conditional branches for error/empty/edit states
function ContractFormFields({
  applyDuration, editingContract, errors, isIndefinite, isSubmitting,
  onClose, phases, register, setIndefinite,
}: ContractFormBodyProps) {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <FormField label="Số hợp đồng" {...register("contract_number")} error={errors.contract_number?.message} placeholder="VD: HD-001" />
        <div className="space-y-2">
          <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="contract_type">
            Loại hợp đồng
            <select className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm" id="contract_type" {...register("contract_type")}>
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
          <FormField label="Ngày hiệu lực" type="date" {...register("start_date")} error={errors.start_date?.message} required />
          <Typography className="text-[10px] text-slate-500 italic px-1" variant="p">Lưu ý: Ngày bắt đầu hợp đồng cũ nhất sẽ tự động cập nhật lại thành Ngày vào làm của nhân viên.</Typography>
        </div>
        <FormField disabled={isIndefinite} label="Ngày hết hạn" type="date" {...register("end_date")} error={errors.end_date?.message} required={!isIndefinite} />
      </div>

      <div className="flex flex-wrap items-center gap-2 pt-1">
        <span className="text-[11px] font-bold text-slate-500 uppercase tracking-widest mr-1">Tính nhanh:</span>
        {DURATION_PRESETS.map((opt) => (
          <Button className="px-2.5 py-1.5 text-[11px] font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-300 h-auto shadow-none border-none normal-case tracking-normal hover:text-slate-800" key={opt.label} onClick={() => { applyDuration(opt.months, opt.years); }} type="button" variant="ghost">{opt.label}</Button>
        ))}
        <Button
          className={cn("px-2.5 py-1.5 text-[11px] font-medium rounded-lg transition-colors h-auto shadow-none border-none normal-case tracking-normal hover:text-slate-800", isIndefinite ? "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-400 ring-1 ring-primary-500/50" : "bg-slate-100 hover:bg-slate-200 text-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-300")}
          onClick={setIndefinite} type="button" variant="ghost">Lâu dài</Button>
      </div>

      {!editingContract ? <PhaseSelector errors={errors} phases={phases} register={register} /> : null}

      {editingContract ? <StatusSelector register={register} /> : null}

      <FormField label="Ghi chú" {...register("notes")} error={errors.notes?.message} placeholder="Ghi chú thêm về hợp đồng..." />

      <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
        <Button disabled={isSubmitting} onClick={onClose} type="button" variant="outline">HỦY BỎ</Button>
        <Button loading={isSubmitting} type="submit" variant="secondary">{editingContract ? "CẬP NHẬT" : "TẠO MỚI"}</Button>
      </div>
    </div>
  );
}

// ── Main Component ──────────────────────────────────────────────────────────

export default function ContractFormModal({ isOpen, onClose, editingContract,employeeId }: ContractFormModalProps) {
  const toast = useToast();
  const { createContract, updateContract } = useEmployeeContracts(employeeId);

  const [{ data: phases }, { data: employments = [] }] = useQueries({
    queries: [
      { queryKey: ["employment-phases"], queryFn: () => fetchEmploymentPhases() },
      { queryKey: ["employee-employments",employeeId], queryFn: () => fetchEmployeeEmployments(employeeId), enabled: !!employeeId && isOpen },
    ],
  });

  const activeEmployment: Employment | undefined = (Array.isArray(employments) ? employments : []).find((e) => e.status === "active");
  const [overlapData, setOverlapData] = useState<CreateContractRequest | null>(null);
  const [isConfirmingOverlap, setIsConfirmingOverlap] = useState(false);

  const { register, handleSubmit, reset, control, setValue, formState: { errors, isSubmitting } } = useForm<ContractFormValues>({
    resolver: standardSchemaResolver(contractSchema),
    defaultValues: { contract_type: "probation", status: "active", start_date: new Date().toISOString().split("T")[0], end_date: "", contract_number: "", notes: "", assign_phase_id: "" },
  });

  const contractType = useWatch({ control, name: "contract_type" });
  const startDateValue = useWatch({ control, name: "start_date" });
  const isIndefinite = contractType === "indefinite";

  const applyDuration = (months?: number, years?: number) => {
    if (!startDateValue) return;
    const start = new Date(startDateValue);
    if (isNaN(start.getTime())) return;
    let end = start;
    if (months) end = addMonths(end, months);
    if (years) end = addYears(end, years);
    end = subDays(end, 1);
    setValue("end_date", format(end, "yyyy-MM-dd"), { shouldValidate: true });
    if (contractType === "indefinite") setValue("contract_type", "fixed_term", { shouldValidate: true });
  };

  useEffect(() => {
    if (!isOpen) return;
    if (editingContract) {
      reset({ contract_type: editingContract.contract_type, status: editingContract.status, start_date: editingContract.start_date, end_date: editingContract.end_date ?? "", contract_number: editingContract.contract_number ?? "", notes: editingContract.notes ?? "" });
    } else {
      reset({ contract_type: "probation", status: "active", start_date: activeEmployment?.start_date ?? new Date().toISOString().split("T")[0], end_date: "", contract_number: "", notes: "", assign_phase_id: "" });
    }
  }, [isOpen, editingContract, reset, activeEmployment?.start_date]);

  // eslint-disable-next-line complexity -- create vs update + overlap conflict resolution are genuine branches
  const onSubmit = async (data: ContractFormValues) => {
    try {
      const payloadDate = isIndefinite ? null : (data.end_date ?? null);
      if (editingContract) { await updateContract.mutateAsync({ contractId: editingContract.id, data: { ...data, end_date: payloadDate } }); toast.success("Đã cập nhật hợp đồng thành công"); onClose(); return; }
      if (!data.assign_phase_id) { toast.error("Vui lòng chọn Cột mốc khởi đầu"); return; }
      const payload: CreateContractRequest = { ...data, employment_id: activeEmployment?.id ?? "", end_date: payloadDate, assign_phase_id: data.assign_phase_id ?? undefined };
      try { await createContract.mutateAsync(payload); toast.success("Đã tạo hợp đồng thành công"); onClose(); }
      catch (err: unknown) { const error = err as Error; if (error.message.includes("overlapping_contract_exists")) { setOverlapData(payload); setIsConfirmingOverlap(true); } else { toast.error(error.message || "Có lỗi xảy ra"); } }
    } catch (e: unknown) { logError(e, { component: 'ContractFormModal.save' }); toast.error("Có lỗi xảy ra"); }
  };

  const handleConfirmOverlap = async () => {
    if (!overlapData) return;
    try { await createContract.mutateAsync({ ...overlapData, auto_close_overlapping: true }); toast.success("Đã tạo hợp đồng thành công và tự động kết thúc hợp đồng cũ"); setIsConfirmingOverlap(false); setOverlapData(null); onClose(); }
    catch (err: unknown) { toast.error((err as Error).message || "Có lỗi xảy ra"); }
  };

  return (
    <>
      <Modal icon={<FileText size={24} />} isOpen={isOpen && !isConfirmingOverlap} maxWidth="md" onClose={onClose} // eslint-disable-line react/jsx-no-leaked-render -- boolean prop, not children
        subtitle={editingContract ? "Cập nhật thông tin hợp đồng nhân sự" : "Tạo hợp đồng mới cho nhân viên"}
        title={editingContract ? "Chỉnh sửa Hợp đồng" : "Tạo Hợp đồng Mới"}>
        <form className="space-y-6 pt-4" noValidate onSubmit={handleSubmit(onSubmit)}>
          <ContractFormFields
            applyDuration={applyDuration} editingContract={editingContract} errors={errors}
            isIndefinite={isIndefinite} isSubmitting={isSubmitting} onClose={onClose} phases={phases}
            register={register} setIndefinite={() => { setValue("contract_type", "indefinite", { shouldValidate: true }); setValue("end_date", "", { shouldValidate: true }); }}
          />
        </form>
      </Modal>
      <OverlapConfirmationModal isOpen={isConfirmingOverlap} isPending={createContract.isPending}
        onClose={() => { setIsConfirmingOverlap(false); }}
        onConfirm={() => { void handleConfirmOverlap(); }}
      />
    </>
  );
}

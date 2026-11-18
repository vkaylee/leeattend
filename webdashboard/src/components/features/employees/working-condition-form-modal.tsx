import type { TFunction } from "i18next";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { HeartHandshake } from "lucide-react";
import { useEffect, useState } from "react";
import { useForm, useWatch, type FieldErrors, type UseFormRegister, type UseFormSetValue } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { z } from "zod";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  createWorkingCondition,
  fetchConditionTypes,
  type ConditionTypeDefinition,
  type Employee,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { cn } from "@/lib/utils";

const EMPTY_CONDITION_TYPES: ConditionTypeDefinition[] = [];

const conditionFormSchema = z
  .object({
    type_: z.string().min(1, { message: "Vui lòng chọn loại điều kiện" }),
    condition_type_id: z.string().nullable().optional(),
    start_date: z.string().min(1, { message: "Ngày bắt đầu là bắt buộc" }),
    end_date: z.string().optional(),
    is_paid: z.boolean(),
    minute_allowance: z.number().min(0).optional(),
  })
  .superRefine((data, ctx) => {
    if (data.start_date && data.end_date && new Date(data.start_date) > new Date(data.end_date)) {
      ctx.addIssue({
        code: "custom",
        message: "Ngày kết thúc phải sau ngày bắt đầu",
        path: ["end_date"],
      });
    }
  });

type ConditionFormValues = z.infer<typeof conditionFormSchema>;

export interface WorkingConditionFormModalProps {
  employeeId?: string;
  isOpen: boolean;
  onClose: () => void;
}

interface EmployeeFieldProps {
  employees: Employee[];
  onChange: (employeeId: string) => void;
  selectedEmployeeId: string;
  t: TFunction;
}

function EmployeeField({ employees, onChange, selectedEmployeeId, t }: EmployeeFieldProps) {
  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="condition-employee-combobox">
        {t("working_conditions.fields.employee", { defaultValue: "Nhân sự" })} *
      </label>
      <Combobox
        clearOnEscape
        data-testid="condition-employee-combobox"
        id="condition-employee-combobox"
        onChange={onChange}
        options={employees.map((employee) => ({
          value: employee.id,
          label: `${employee.full_name}${employee.employee_id_number ? ` (${employee.employee_id_number})` : ""}`,
          description: employee.department_name ?? employee.job_title ?? undefined,
        }))}
        placeholder={t("working_conditions.placeholders.select_employee", { defaultValue: "Chọn nhân sự áp dụng..." })}
        value={selectedEmployeeId}
      />
    </div>
  );
}

interface ConditionTypeFieldProps {
  currentLang: string;
  errors: FieldErrors<ConditionFormValues>;
  register: UseFormRegister<ConditionFormValues>;
  setValue: UseFormSetValue<ConditionFormValues>;
  t: TFunction;
  typeDefs: ConditionTypeDefinition[];
}

function ConditionTypeField({ currentLang, errors, register, setValue, t, typeDefs }: ConditionTypeFieldProps) {
  return (
    <div className="space-y-2">
      <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="condition-type-select">
        {t("working_conditions.fields.type", { defaultValue: "Loại điều kiện" })} *
      </label>
      <select
        className="w-full h-11 px-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold text-slate-800 dark:text-slate-100"
        id="condition-type-select"
        {...register("type_")}
        onChange={(event) => {
          const typeCode = event.target.value;
          setValue("type_", typeCode);
          setValue("condition_type_id", typeDefs.find((definition) => definition.code === typeCode)?.id ?? null);
        }}
      >
        {typeDefs.map((definition) => {
          const name = definition.name_i18n[currentLang] || definition.name_i18n.en || definition.code;
          return <option key={definition.id} value={definition.code}>{name} ({definition.code})</option>;
        })}
      </select>
      {!!errors.type_?.message && <Typography className="text-sm font-semibold text-rose-500 mt-1" variant="p">{errors.type_.message}</Typography>}
    </div>
  );
}

interface PaymentRegimeFieldProps {
  isPaid: boolean;
  onChange: (isPaid: boolean) => void;
  t: TFunction;
}

function PaymentRegimeField({ isPaid, onChange, t }: PaymentRegimeFieldProps) {
  return (
    <div className="space-y-2">
      <span className="text-[11px] font-bold text-slate-500 uppercase tracking-widest block">{t("working_conditions.fields.payment_regime", { defaultValue: "Chế độ hưởng lương" })}</span>
      <div className="flex gap-3 p-1 bg-slate-100 dark:bg-slate-800/50 rounded-2xl">
        <Button className={cn("flex-1 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider transition-all", isPaid ? "bg-white dark:bg-slate-700 text-emerald-600 dark:text-emerald-300 shadow-xs" : "text-slate-400")} onClick={() => { onChange(true); }} type="button" variant="ghost">
          {t("working_conditions.paid", { defaultValue: "Có hưởng lương" })}
        </Button>
        <Button className={cn("flex-1 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider transition-all", !isPaid ? "bg-white dark:bg-slate-700 text-slate-700 dark:text-slate-200 shadow-xs" : "text-slate-400")} onClick={() => { onChange(false); }} type="button" variant="ghost">
          {t("working_conditions.unpaid", { defaultValue: "Không hưởng lương" })}
        </Button>
      </div>
    </div>
  );
}

export default function WorkingConditionFormModal({ isOpen, onClose, employeeId }: WorkingConditionFormModalProps) {
  const { t, i18n } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const currentLang = i18n.language.split("-")[0];
  const { employees } = useAllEmployees({ enabled: isOpen && !employeeId });
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>(employeeId ?? "");
  const [prevOpen, setPrevOpen] = useState(isOpen);

  if (isOpen !== prevOpen) {
    setPrevOpen(isOpen);
    if (isOpen) setSelectedEmployeeId(employeeId ?? "");
  }

  const { data: typeDefs } = useQuery<ConditionTypeDefinition[]>({
    queryKey: createQueryKey("condition-types"),
    queryFn: () => fetchConditionTypes(true),
    enabled: isOpen,
  });
  const definitions = typeDefs ?? EMPTY_CONDITION_TYPES;
  const {
    register,
    handleSubmit,
    control,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<ConditionFormValues>({
    resolver: standardSchemaResolver(conditionFormSchema),
    defaultValues: {
      type_: "",
      condition_type_id: null,
      start_date: new Date().toISOString().split("T")[0],
      end_date: "",
      is_paid: true,
      minute_allowance: 0,
    },
  });
  const isPaid = useWatch({ control, name: "is_paid", defaultValue: true });

  useEffect(() => {
    if (!isOpen) return;
    const firstType = definitions.find((definition) => definition.is_active);
    reset({ type_: firstType?.code ?? "", condition_type_id: firstType?.id ?? null, start_date: new Date().toISOString().split("T")[0], end_date: "", is_paid: true, minute_allowance: 0 });
  }, [definitions, isOpen, reset]);

  const onSubmit = async (values: ConditionFormValues) => {
    try {
      const targetEmployeeId = employeeId || selectedEmployeeId;
      if (!targetEmployeeId) {
        toast.error("Vui lòng chọn nhân sự");
        return;
      }
      const matchingDefinition = definitions.find((definition) => definition.code === values.type_);
      await createWorkingCondition(targetEmployeeId, {
        type_: values.type_,
        condition_type_id: matchingDefinition?.id ?? null,
        start_date: values.start_date,
        end_date: values.end_date ? values.end_date : null,
        is_paid: values.is_paid,
        minute_allowance: values.minute_allowance ?? 0,
      } as Parameters<typeof createWorkingCondition>[1]);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("working-conditions") });
      toast.success(t("working_conditions.create_success", { defaultValue: "Đã gán điều kiện làm việc thành công" }));
      onClose();
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : "Có lỗi xảy ra khi tạo điều kiện làm việc");
    }
  };

  return (
    <Modal icon={<HeartHandshake size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("working_conditions.modal_subtitle", { defaultValue: "Gán chế độ đặc thù (con nhỏ, thai sản, độc hại, remote...) cho nhân sự" })} title={t("working_conditions.modal_title", { defaultValue: "Gán Điều kiện Làm việc" })}>
      <form className="space-y-6 pt-4" onSubmit={handleSubmit(onSubmit)}>
        <div className="space-y-4">
          {!employeeId && <EmployeeField employees={employees} onChange={setSelectedEmployeeId} selectedEmployeeId={selectedEmployeeId} t={t} />}
          <ConditionTypeField currentLang={currentLang} errors={errors} register={register} setValue={setValue} t={t} typeDefs={definitions} />
          <div className="grid grid-cols-2 gap-4">
            <FormField error={errors.start_date?.message} label={t("working_conditions.fields.start_date", { defaultValue: "Ngày bắt đầu" })} required type="date" {...register("start_date")} />
            <FormField error={errors.end_date?.message} label={t("working_conditions.fields.end_date", { defaultValue: "Ngày kết thúc" })} type="date" {...register("end_date")} />
          </div>
          <PaymentRegimeField isPaid={isPaid} onChange={(paid) => { setValue("is_paid", paid); }} t={t} />
        </div>
        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button disabled={isSubmitting} onClick={onClose} type="button" variant="outline">{t("common.cancel", { defaultValue: "HỦY BỎ" })}</Button>
          <Button loading={isSubmitting} type="submit" variant="secondary">{t("common.save", { defaultValue: "LƯU ĐIỀU KIỆN" })}</Button>
        </div>
      </form>
    </Modal>
  );
}

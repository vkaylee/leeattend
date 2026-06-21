import type { TFunction } from "i18next";

import { Calendar, CalendarDays, FileText, Info, Loader2, Moon, PlayCircle, Sun, Timer, X } from "lucide-react";
import React from "react";

import type { Employee, EmployeeLeaveBalance, LeaveDurationType, LeaveType } from "@/lib/api/types";

import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

const getLeaveTypeIcon = (code: string) => {
  const c = code.toLowerCase();
  if (c.includes("vacation") || c.includes("annual")) return <Calendar size={14} />;
  if (c.includes("sick")) return <FileText size={14} />;
  if (c.includes("personal")) return <CalendarDays size={14} />;
  return <PlayCircle size={14} />;
};

function LeaveBalancesPanel({ balances, selectedEmpId, t }: { balances: EmployeeLeaveBalance[]; selectedEmpId: string; t: TFunction }) {
  if (!selectedEmpId || balances.length === 0) return null;
  return (
    <div className="bg-primary-50 dark:bg-primary-950/50 p-8 rounded-[2.5rem] border border-primary-100 dark:border-primary-800/50 grid grid-cols-2 gap-8 shadow-inner">
      {balances.map(b => (
        <div className="space-y-2" key={b.leave_type_id ?? b.leave_type_code}>
          <Typography className="text-[9px] font-bold uppercase tracking-[0.25em] text-primary-600 dark:text-primary-400 opacity-70 leading-none" variant="p">{(b.leave_type_name ?? b.leave_type_code ?? "Unknown").toUpperCase()}</Typography>
          <Typography className="text-base font-bold text-primary-700 dark:text-primary-300 tracking-tighter uppercase" variant="p">{t("leave_requests.remaining")}: <span className="text-primary-600 dark:text-primary-400 text-2xl font-mono ml-1">{b.remaining}</span><span className="text-[10px] opacity-40 ml-1">/ {b.entitlement + b.carried_over}</span></Typography>
        </div>
      ))}
    </div>
  );
}

function LeaveDurationSelector({
  durationType,
  setDurationType,
  setEndDate,
  startDate,
  t
}: {
  durationType: LeaveDurationType;
  setDurationType: (d: LeaveDurationType) => void;
  setEndDate: (d: string) => void;
  startDate: string;
  t: TFunction;
}) {
  return (
    <div className="space-y-3 px-1">
      <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1">{t("leave_requests.duration.label")}</label>
      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1 rounded-2xl border border-primary-100 dark:border-primary-800/50 shadow-inner">
        {([
          { value: "full_day" as LeaveDurationType, label: t("leave_requests.duration.full_day"), icon: <Calendar size={12} /> },
          { value: "first_half" as LeaveDurationType, label: t("leave_requests.duration.first_half"), icon: <Sun size={12} /> },
          { value: "second_half" as LeaveDurationType, label: t("leave_requests.duration.second_half"), icon: <Moon size={12} /> },
          { value: "custom_hours" as LeaveDurationType, label: t("leave_requests.duration.custom_hours"), icon: <Timer size={12} /> },
        ]).map(opt => (
          <Button
            className={cn(
              "flex-1 flex items-center justify-center gap-1.5 px-3 py-2.5 rounded-xl text-[9px] font-bold uppercase tracking-wider transition-all border-none shadow-none h-auto w-auto",
              durationType === opt.value
                ? "bg-white dark:bg-slate-900 text-primary-700 dark:text-primary-300 shadow-sm border border-primary-100 dark:border-primary-800/50"
                : "text-primary-400 hover:text-primary-600 dark:text-primary-500 dark:hover:text-primary-300"
            )}
            key={opt.value}
            onClick={() => {
              setDurationType(opt.value);
              if (opt.value !== "full_day" && startDate) {
                setEndDate(startDate);
              }
            }}
            variant="ghost"
          >
            {opt.icon} {opt.label}
          </Button>
        ))}
      </div>
      {durationType !== "full_day" && durationType !== "custom_hours" && (
        <Typography className="text-[10px] text-primary-400 italic ml-1 flex items-center gap-1" variant="p"><Info size={10} /> {t("leave_requests.duration.half_day_note")}</Typography>
      )}
    </div>
  );
}

function CustomHoursFields({
  endTime,
  setEndTime,
  setStartTime,
  startTime,
  t
}: {
  endTime: string;
  setEndTime: (t: string) => void;
  setStartTime: (t: string) => void;
  startTime: string;
  t: TFunction;
}) {
  return (
    <div className="grid grid-cols-2 gap-8 animate-in fade-in slide-in-from-top-2 duration-300">
      <FormField id="start-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_from")} onChange={e => { setStartTime(e.target.value); }} required type="time" value={startTime} />
      <FormField id="end-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_to")} onChange={e => { setEndTime(e.target.value); }} required type="time" value={endTime} />
    </div>
  );
}

function EmployeeSelectorField({
  canSubmitOnBehalf,
  hasPermission,
  behalfLoading,
  selectedEmpId,
  setSelectedEmpId,
  behalfOptions,
  employee,
  t
}: {
  canSubmitOnBehalf: boolean;
  hasPermission: boolean;
  behalfLoading: boolean;
  selectedEmpId: string;
  setSelectedEmpId: (id: string) => void;
  behalfOptions: { value: string; label: string }[];
  employee: Employee | undefined;
  t: TFunction;
}) {
  if (canSubmitOnBehalf) {
    return (
      <Combobox
        className="h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 shadow-sm"
        id="leave-emp"
        onChange={setSelectedEmpId}
        options={behalfOptions}
        placeholder={t("common.select_employee")}
        value={selectedEmpId}
      />
    );
  }
  if (hasPermission && behalfLoading) {
    return (
      <div className="h-14 rounded-2xl bg-slate-100 dark:bg-slate-800/50 border border-primary-100 dark:border-primary-800/50 px-5 flex items-center gap-2 text-primary-400 text-sm">
        <Loader2 className="animate-spin" size={14} />
        {t("common.loading")}
      </div>
    );
  }
  if (hasPermission) {
    return (
      <div className="h-14 rounded-2xl bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 px-5 flex items-center gap-2 text-amber-700 dark:text-amber-400 text-sm">
        <Info size={14} />
        {t("behalf.no_assignments_configured", "Chưa có cấu hình behalf assignment")}
      </div>
    );
  }
  return (
    <div className="h-14 rounded-2xl bg-slate-100 dark:bg-slate-800/50 border border-primary-100 dark:border-primary-800/50 px-5 flex items-center text-primary-700 dark:text-primary-350 font-semibold text-sm">
      {employee?.full_name ?? "Loading..."}
    </div>
  );
}

export interface CreateLeaveRequestFormProps {
  attachmentFileIds: string[];
  balances: EmployeeLeaveBalance[];
  behalfEmployees: Employee[];
  behalfLoading: boolean;
  canSubmitOnBehalf: boolean;
  durationType: LeaveDurationType;
  employee: Employee | undefined;
  endDate: string;
  endTime: string;
  /** Whether user has RBAC permission (but maybe no behalf config) */
  hasPermission: boolean;
  isSubmitting: boolean;
  leaveTypeId: string;
  leaveTypes: LeaveType[];
  onCancel: () => void;
  onSubmit: (e: React.SyntheticEvent<HTMLFormElement>) => void;
  ownerId: string;
  selectedEmpId: string;
  setAttachmentFileIds: (ids: string[]) => void;
  setDurationType: (d: LeaveDurationType) => void;
  setEndDate: (d: string) => void;
  setEndTime: (t: string) => void;
  setLeaveTypeId: (id: string) => void;
  setSelectedEmpId: (id: string) => void;
  setStartDate: (d: string) => void;
  setStartTime: (t: string) => void;
  startDate: string;
  startTime: string;
  t: TFunction;
  wsId: string;
}

export function CreateLeaveRequestForm({
  balances,
  behalfEmployees,
  behalfLoading,
  canSubmitOnBehalf,
  durationType,
  employee,
  endDate,
  endTime,
  hasPermission,
  isSubmitting,
  leaveTypeId,
  leaveTypes,
  onCancel,
  onSubmit,
  selectedEmpId,
  setDurationType,
  setEndDate,
  setEndTime,
  setLeaveTypeId,
  setSelectedEmpId,
  setStartDate,
  setStartTime,
  startDate,
  startTime,
  t,
  wsId,
  ownerId,
  attachmentFileIds,
  setAttachmentFileIds,
}: CreateLeaveRequestFormProps) {
  const empIdToUse = canSubmitOnBehalf ? selectedEmpId : (employee?.id ?? "");

  const behalfOptions = behalfEmployees.map(emp => ({
    value: emp.id,
    label: `${emp.full_name} (${emp.employee_id_number})`,
  }));

  return (
    <form className="space-y-8 pt-4" onSubmit={onSubmit}>
      <div className="space-y-3 px-1">
        <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1" htmlFor="leave-emp">{t("common.employee")}</label>
        <EmployeeSelectorField
          behalfLoading={behalfLoading}
          behalfOptions={behalfOptions}
          canSubmitOnBehalf={canSubmitOnBehalf}
          employee={employee}
          hasPermission={hasPermission}
          selectedEmpId={selectedEmpId}
          setSelectedEmpId={setSelectedEmpId}
          t={t}
        />
      </div>
      <LeaveBalancesPanel balances={balances} selectedEmpId={empIdToUse} t={t} />
      <div className="space-y-3 px-1">
        <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1" htmlFor="leave-type">{t("common.type")}</label>
        <Combobox 
          className="bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 h-14" 
          id="leave-type" 
          onChange={setLeaveTypeId} 
          options={leaveTypes.map(tOption => ({ value: tOption.id, label: tOption.name.toUpperCase(), icon: getLeaveTypeIcon(tOption.code) }))} 
          value={leaveTypeId} 
        />
      </div>
      <div className="grid grid-cols-2 gap-8">
        <FormField id="start-date" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("common.from_date")} name="start_date" onChange={e => { setStartDate(e.target.value); if (durationType !== "full_day") setEndDate(e.target.value); }} required type="date" value={startDate} />
        <FormField disabled={durationType !== "full_day"} id="end-date" inputClassName={cn("text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50", durationType !== "full_day" && "opacity-50 cursor-not-allowed")} label={t("common.to_date")} name="end_date" onChange={e => { setEndDate(e.target.value); }} required type="date" value={endDate} />
      </div>

      {/* Duration Type Segmented Buttons */}
      <LeaveDurationSelector
        durationType={durationType}
        setDurationType={setDurationType}
        setEndDate={setEndDate}
        startDate={startDate}
        t={t}
      />

      {/* Time Pickers for Custom Hours */}
      {durationType === "custom_hours" && (
        <CustomHoursFields
          endTime={endTime}
          setEndTime={setEndTime}
          setStartTime={setStartTime}
          startTime={startTime}
          t={t}
        />
      )}

      <FormField id="leave-reason" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("common.reason")} name="reason" placeholder={t("leave_requests.modal.reason_placeholder")} rows={3} type="textarea" />
      
      <div className="space-y-3 px-1">
        <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1">
          {t("leave_requests.modal.attachment", "ATTACHMENT (PDF ONLY)")}
        </label>
        <FileUpload
          accept="application/pdf"
          category="LeaveAttachment"
          maxSizeMB={20}
          onUploadSuccess={(file) => {
            setAttachmentFileIds([...attachmentFileIds, file.id]);
          }}
          ownerId={ownerId}
          ownerType="LeaveRequest"
          workspaceId={wsId}
        />
        {attachmentFileIds.length > 0 && (
          <div className="mt-3 space-y-2">
            {attachmentFileIds.map((id, index) => (
              <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/50 rounded-xl border border-primary-100 dark:border-primary-800/50 text-xs animate-in slide-in-from-top-1" key={id}>
                <span className="font-mono text-primary-750 dark:text-primary-300">Document #{index + 1} ({id})</span>
                <Button
                  className="p-1 h-6 w-6 text-rose-600 hover:bg-rose-100 dark:hover:bg-rose-900/50 rounded-lg transition-colors border-none"
                  onClick={() => {
                    setAttachmentFileIds(attachmentFileIds.filter(fId => fId !== id));
                  }}
                  type="button"
                  variant="ghost"
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
        <Button onClick={onCancel} size="lg" type="button" variant="outline">{t("common.cancel").toUpperCase()}</Button>
        <Button disabled={isSubmitting} size="lg" type="submit" variant="primary">{t("leave_requests.modal.submit_btn").toUpperCase()}</Button>
      </div>
    </form>
  );
}

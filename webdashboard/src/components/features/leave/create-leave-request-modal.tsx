import type { TFunction } from "i18next";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { CalendarDays, Calendar, FileText, PlayCircle, Sun, Moon, Timer, Info } from "lucide-react";
import React, { useState } from "react";

import type { LeaveDurationType, LeaveType } from "@/lib/api/types";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchEmployeeLeaveBalances, submitLeaveRequest } from "@/lib/api";
import { cn } from "@/lib/utils";

export interface CreateLeaveRequestModalProps {
  isOpen: boolean;
  leaveTypes: LeaveType[];
  onClose: () => void;
  t: TFunction;
  wsId: string;
}

const getLeaveTypeIcon = (code: string) => {
  const c = code.toLowerCase();
  if (c.includes("vacation") || c.includes("annual")) return <Calendar size={14} />;
  if (c.includes("sick")) return <FileText size={14} />;
  if (c.includes("personal")) return <CalendarDays size={14} />;
  return <PlayCircle size={14} />;
};

import type { EmployeeLeaveBalance } from "@/lib/api";

function LeaveBalancesPanel({ balances, selectedEmpId, t }: { balances: EmployeeLeaveBalance[]; selectedEmpId: string; t: TFunction }) {
  if (!selectedEmpId || balances.length === 0) return null;
  return (
    <div className="bg-primary-50 dark:bg-primary-950/50 p-8 rounded-[2.5rem] border border-primary-100 dark:border-primary-800/50 grid grid-cols-2 gap-8 shadow-inner">
      {balances.map(b => (
        <div className="space-y-2" key={b.leave_type_id || b.leave_type_code}>
          <Typography className="text-[9px] font-bold uppercase tracking-[0.25em] text-primary-600 dark:text-primary-400 opacity-70 leading-none" variant="p">{(b.leave_type_name || b.leave_type_code || "Unknown").toUpperCase()}</Typography>
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
          <button
            className={cn(
              "flex-1 flex items-center justify-center gap-1.5 px-3 py-2.5 rounded-xl text-[9px] font-bold uppercase tracking-wider transition-all",
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
            type="button"
          >
            {opt.icon} {opt.label}
          </button>
        ))}
      </div>
      {durationType !== "full_day" && durationType !== "custom_hours" && (
        <Typography className="text-[10px] text-primary-400 italic ml-1 flex items-center gap-1" variant="p"><Info size={10} /> {t("leave_requests.duration.half_day_note")}</Typography>
      )}
    </div>
  );
}

export function CreateLeaveRequestModal({ isOpen, onClose, wsId, leaveTypes, t }: CreateLeaveRequestModalProps) {
  const toast = useToast();
  const queryClient = useQueryClient();

  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveTypeId, setLeaveTypeId] = useState<string>("");
  const [durationType, setDurationType] = useState<LeaveDurationType>("full_day");
  const [startTime, setStartTime] = useState<string>("");
  const [endTime, setEndTime] = useState<string>("");
  const [startDate, setStartDate] = useState<string>("");
  const [endDate, setEndDate] = useState<string>("");

  const handleClose = () => {
    setSelectedEmpId(""); setDurationType("full_day");
    setStartTime(""); setEndTime(""); setStartDate(""); setEndDate("");
    onClose();
  };

  const { data: balances = [] } = useQuery({ 
    queryKey: ["leave-balances", wsId, selectedEmpId], 
    queryFn: () => fetchEmployeeLeaveBalances(wsId, selectedEmpId), 
    enabled: !!wsId && !!selectedEmpId && isOpen 
  });

  if (leaveTypes.length > 0 && !leaveTypeId) {
    setLeaveTypeId(leaveTypes[0].id);
  }

  const submitMutation = useMutation({
    mutationFn: (data: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string }) => submitLeaveRequest(wsId, data),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); 
      handleClose();
      toast.success(t("common.success")); 
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault(); 
    const formData = new FormData(e.currentTarget);
    if (!leaveTypeId) { toast.error(t("leave_requests.invalid_leave_type")); return; }
 
    const payload: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string } = {
      employee_id: selectedEmpId,
      leave_type_id: leaveTypeId,
      start_date: startDate || (formData.get("start_date") as string),
      end_date: endDate || (formData.get("end_date") as string),
      reason: formData.get("reason") as string,
      duration_type: durationType,
    };
    if (durationType === "custom_hours" && startTime && endTime) {
      payload.start_time = startTime;
      payload.end_time = endTime;
    }
    submitMutation.mutate(payload);
  };

  return (
    <Modal icon={<CalendarDays size={24} />} isOpen={isOpen} maxWidth="md" onClose={handleClose} subtitle={t("leave_requests.modal.submit_subtitle")} title={t("leave_requests.modal.submit_title")}>
      <form className="space-y-8 pt-4" onSubmit={handleSubmit}>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest ml-1" htmlFor="leave-emp">{t("common.employee")}</label>
          <EmployeeSearch className="h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border-primary-100 dark:border-primary-800/50 shadow-sm" id="leave-emp" onChange={(value) => { setSelectedEmpId(Array.isArray(value) ? value[0] || "" : value); }} value={selectedEmpId} workspaceId={wsId} />
        </div>
        <LeaveBalancesPanel balances={balances} selectedEmpId={selectedEmpId} t={t} />
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
          <div className="grid grid-cols-2 gap-8 animate-in fade-in slide-in-from-top-2 duration-300">
            <FormField id="start-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_from")} onChange={e => { setStartTime(e.target.value); }} required type="time" value={startTime} />
            <FormField id="end-time" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("leave_requests.duration.time_to")} onChange={e => { setEndTime(e.target.value); }} required type="time" value={endTime} />
          </div>
        )}

        <FormField id="leave-reason" inputClassName="text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/50" label={t("common.reason")} name="reason" placeholder={t("leave_requests.modal.reason_placeholder")} rows={3} type="textarea" />
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={handleClose} size="lg" type="button" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button size="lg" type="submit" variant="primary">{t("leave_requests.modal.submit_btn").toUpperCase()}</Button>
        </div>
      </form>
    </Modal>
  );
}

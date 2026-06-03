import type { TFunction } from "i18next";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CalendarDays } from "lucide-react";
import React, { useState } from "react";

import type { LeaveDurationType, LeaveType } from "@/lib/api/types";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchEmployeeLeaveBalances, submitLeaveRequest } from "@/lib/api";

import { CreateLeaveRequestForm } from "./create-leave-request-form";

export interface CreateLeaveRequestModalProps {
  isOpen: boolean;
  leaveTypes: LeaveType[];
  onClose: () => void;
  t: TFunction;
  wsId: string;
}

export function CreateLeaveRequestModal({ isOpen, onClose, wsId, leaveTypes, t }: CreateLeaveRequestModalProps) {
  const toast = useToast();
  const queryClient = useQueryClient();
  const { hasAnyPermission } = useMyPermissions();
  const { employee } = useMyProfile();

  const canSubmitOnBehalf = hasAnyPermission(
    "leave:requests:submit_on_behalf",
    "leave:requests:approve",
    "leave:approve:l1",
    "leave:approve:l2"
  );

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

  const empIdToUse = canSubmitOnBehalf ? selectedEmpId : (employee?.id ?? "");

  const { data: balances = [] } = useQuery({ 
    queryKey: ["leave-balances", wsId, empIdToUse], 
    queryFn: () => fetchEmployeeLeaveBalances(wsId, empIdToUse), 
    enabled: !!wsId && !!empIdToUse && isOpen 
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
    if (!leaveTypeId) { toast.error(t("leave_requests.invalid_leave_type")); return; }
    if (!empIdToUse) { toast.error("Please select an employee"); return; }
 
    const payload: { employee_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string } = {
      employee_id: empIdToUse,
      leave_type_id: leaveTypeId,
      start_date: startDate,
      end_date: endDate,
      reason: new FormData(e.currentTarget).get("reason") as string,
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
      <CreateLeaveRequestForm
        balances={balances}
        canSubmitOnBehalf={canSubmitOnBehalf}
        durationType={durationType}
        employee={employee}
        endDate={endDate}
        endTime={endTime}
        isSubmitting={submitMutation.isPending}
        leaveTypeId={leaveTypeId}
        leaveTypes={leaveTypes}
        onCancel={handleClose}
        onSubmit={handleSubmit}
        selectedEmpId={selectedEmpId}
        setDurationType={setDurationType}
        setEndDate={setEndDate}
        setEndTime={setEndTime}
        setLeaveTypeId={setLeaveTypeId}
        setSelectedEmpId={setSelectedEmpId}
        setStartDate={setStartDate}
        setStartTime={setStartTime}
        startDate={startDate}
        startTime={startTime}
        t={t}
        wsId={wsId}
      />
    </Modal>
  );
}

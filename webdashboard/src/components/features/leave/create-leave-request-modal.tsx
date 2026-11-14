import type { TFunction } from "i18next";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CalendarDays } from "lucide-react";
import React, { useState } from "react";

import type { LeaveDurationType, LeaveType } from "@/lib/api/types";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useBehalfEmployees } from "@/hooks/use-behalf-employees";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchEmployeeLeaveBalances, submitLeaveRequest, fetchEmployeeEmployments } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { maybeNull } from "@/types/api-overrides";
import { generateUUID } from "@/utils/uuid";

import { CreateLeaveRequestForm } from "./create-leave-request-form";


export interface CreateLeaveRequestModalProps {
  isOpen: boolean;
  leaveTypes: LeaveType[];
  onClose: () => void;
  t: TFunction;
}

  // eslint-disable-next-line complexity
export function CreateLeaveRequestModal({ isOpen, onClose,leaveTypes, t }: CreateLeaveRequestModalProps) {
  const toast = useToast();
  const queryClient = useQueryClient();
  const { hasAnyPermission } = useMyPermissions();
  const { employee } = useMyProfile();
  const { employees: behalfEmployees, hasAssignments: hasBehalfAssignments, isLoading: behalfLoading } = useBehalfEmployees("leave");

  const hasPermission = hasAnyPermission(
    "leave:requests:submit_on_behalf",
    "leave:requests:approve",
    "leave:approve:l1",
    "leave:approve:l2"
  );
  // Must have RBAC permission AND behalf assignments
  const canSubmitOnBehalf = hasPermission && hasBehalfAssignments;

  const [selectedEmpId, setSelectedEmpId] = useState<string>("");
  const [leaveTypeId, setLeaveTypeId] = useState<string>("");
  const [durationType, setDurationType] = useState<LeaveDurationType>("full_day");
  const [startTime, setStartTime] = useState<string>("");
  const [endTime, setEndTime] = useState<string>("");
  const [startDate, setStartDate] = useState<string>("");
  const [endDate, setEndDate] = useState<string>("");
  const [attachmentFileIds, setAttachmentFileIds] = useState<string[]>([]);
  const [ownerId] = useState<string>(() => generateUUID());

  const handleClose = () => {
    setSelectedEmpId(""); setDurationType("full_day");
    setStartTime(""); setEndTime(""); setStartDate(""); setEndDate("");
    setAttachmentFileIds([]);
    onClose();
  };

  const empIdToUse = canSubmitOnBehalf ? selectedEmpId : (employee?.id ?? "");

  const { data: balances = [] } = useQuery({ queryKey: createQueryKey("leave-balances", empIdToUse), queryFn: () => fetchEmployeeLeaveBalances(empIdToUse),
  enabled: !!empIdToUse && isOpen });

  const { data: employments = [] } = useQuery({ queryKey: createQueryKey("employee-employments", empIdToUse), queryFn: () => fetchEmployeeEmployments(empIdToUse),
  enabled: !!empIdToUse && isOpen, });
  const activeEmployment = employments.find(e => e.status === "active") ?? employments[0];

  if (leaveTypes.length > 0 && !leaveTypeId) {
    setLeaveTypeId(leaveTypes[0].id);
  }

  const submitMutation = useMutation({
    mutationFn: (data: { employment_id: string; leave_type_id: string; start_date: string; end_date: string; reason?: string; duration_type?: LeaveDurationType; start_time?: string; end_time?: string }) => submitLeaveRequest(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey("leave-requests") });
      handleClose();
      toast.success(t("common.success"));
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!leaveTypeId) { toast.error(t("leave_requests.invalid_leave_type")); return; }
    if (!empIdToUse) { toast.error("Please select an employee"); return; }
    if (!maybeNull(activeEmployment)) { toast.error(t("leave_requests.no_employment_info", "Không tìm thấy thông tin công việc")); return; }

    const payload: {
      employment_id: string;
      leave_type_id: string;
      start_date: string;
      end_date: string;
      reason?: string;
      duration_type?: LeaveDurationType;
      start_time?: string;
      end_time?: string;
      attachment_file_ids?: string[];
    } = {
      employment_id: activeEmployment.id,
      leave_type_id: leaveTypeId,
      start_date: startDate,
      end_date: endDate,
      reason: new FormData(e.currentTarget).get("reason") as string,
      duration_type: durationType,
      attachment_file_ids: attachmentFileIds,
    };
    if (durationType === "custom_hours" && startTime && endTime) {
      payload.start_time = startTime;
      payload.end_time = endTime;
    }
    submitMutation.mutate(payload);
  };

  return (
    <Modal icon={<CalendarDays size={24} />} isOpen={isOpen} maxWidth="md" onClose={handleClose} subtitle={t("leave_requests.modal.submit_subtitle")} title={t("leave_requests.modal.submit_title")}>
      <CreateLeaveRequestForm attachmentFileIds={attachmentFileIds}
      balances={balances}
      behalfEmployees={behalfEmployees}
      behalfLoading={behalfLoading}
      canSubmitOnBehalf={canSubmitOnBehalf}
      durationType={durationType}
      employee={employee}
      endDate={endDate}
      endTime={endTime}
      hasPermission={hasPermission}
      isSubmitting={submitMutation.isPending}
      leaveTypeId={leaveTypeId}
      leaveTypes={leaveTypes}
      onCancel={handleClose}
      onSubmit={handleSubmit}
      ownerId={ownerId}
      selectedEmpId={selectedEmpId}
      setAttachmentFileIds={setAttachmentFileIds}
      setDurationType={setDurationType}
      setEndDate={setEndDate}
      setEndTime={setEndTime}
      setLeaveTypeId={setLeaveTypeId}
      setSelectedEmpId={setSelectedEmpId}
      setStartDate={setStartDate}
      setStartTime={setStartTime}
      startDate={startDate}
      startTime={startTime}
      t={t}  />
    </Modal>
  );
}

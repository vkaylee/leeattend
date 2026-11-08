import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { History } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { adjustLeaveBalance, fetchLeaveHistory, fetchLeaveTypes } from "@/lib/api";
import { cn } from "@/lib/utils";

import Button from "../ui/button";
import Modal from "../ui/modal";
import { useToast } from "../ui/toast";
import { AdjustmentForm } from "./history-modal-components/adjustment-form";
import { HistoryTable } from "./history-modal-components/history-table";

interface LeaveHistoryModalProps {
  employeeId: string;
  employeeName: string;
  isOpen: boolean;
  leaveTypeId?: string;
  leaveTypeName?: string;
  onClose: () => void;
}

export function LeaveHistoryModal({employeeId, employeeName, leaveTypeId, leaveTypeName, isOpen, onClose }: LeaveHistoryModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const [activeTab, setActiveTab] = useState("history");

  const { data: history = [], isLoading } = useQuery({ 
    queryKey: ["leave-history",employeeId, leaveTypeId], 
    queryFn: () => fetchLeaveHistory(employeeId, leaveTypeId), 
    enabled: isOpen 
  });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types"], queryFn: () => fetchLeaveTypes(), 
  enabled: isOpen });

  const adjustMutation = useMutation({
    mutationFn: (data: { leave_type_id: string; amount: number; reason: string; transaction_date: string }) => adjustLeaveBalance(employeeId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-history"] });
      void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
      toast.success(t("leave_balances.history.messages.adjust_success")); 
      setActiveTab("history");
    },
  });

  const [adjustForm, setAdjustForm] = useState({ 
    type: "add", 
    leave_type_id: leaveTypeId ?? "", 
    amount: "", 
    reason: "", 
    date: format(new Date(), 'yyyy-MM-dd') 
  });

  const handleAdjustSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault(); 
    const amount = parseFloat(adjustForm.amount);
    if (isNaN(amount) || amount <= 0) return;
    
    const typeId = adjustForm.leave_type_id || leaveTypes.find(t => t.code === "vacation")?.id;
    if (!typeId) { 
      toast.error("Please select a leave type"); 
      return; 
    }
    
    adjustMutation.mutate({ 
      leave_type_id: typeId, 
      amount: adjustForm.type === "add" ? amount : -amount, 
      reason: adjustForm.reason, 
      transaction_date: adjustForm.date 
    });
  };

  return (
    <Modal 
      icon={<History size={24} />} 
      isOpen={isOpen} 
      maxWidth="2xl" 
      onClose={onClose} 
      subtitle={t("leave_balances.history.subtitle")} 
      title={`${t("leave_balances.history.title")}: ${employeeName}${leaveTypeName ? ` — ${leaveTypeName}` : ''}`}
    >
      <div className="space-y-8 pt-4 animate-in fade-in duration-500">
        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <Button 
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all h-auto", activeTab === 'history' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} 
            onClick={() => { setActiveTab("history"); }}
            variant="ghost"
          >
            {t("leave_balances.history.tab_history")}
          </Button>
          <Button 
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all h-auto", activeTab === 'adjust' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} 
            onClick={() => { setActiveTab("adjust"); }}
            variant="ghost"
          >
            {t("leave_balances.history.tab_adjust")}
          </Button>
        </div>

        {activeTab === "history" ? (
          <HistoryTable 
            formatDate={formatDate} 
            history={history} 
            isLoading={isLoading} 
            t={t} 
          />
        ) : (
          <AdjustmentForm 
            adjustForm={adjustForm} 
            isPending={adjustMutation.isPending} 
            leaveTypes={leaveTypes} 
            onCancel={onClose} 
            onSubmit={handleAdjustSubmit} 
            setAdjustForm={setAdjustForm} 
            t={t} 
          />
        )}
      </div>
    </Modal>
  );
}
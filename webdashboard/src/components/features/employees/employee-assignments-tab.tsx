import { Briefcase, Pencil, Plus, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { AssignmentStatus, EmployeeAssignment } from "@/lib/api";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { useEmployeeAssignments, useDeleteEmployeeAssignment } from "@/hooks/use-employee-assignments";
import { cn } from "@/lib/utils";

import EmployeeAssignmentFormModal from "./employee-assignment-form-modal";

interface EmployeeAssignmentsTabProps {
  employeeId: string;
  formatDate: (date: string) => string;
}

const getStatusColor = (status: AssignmentStatus) => {
  switch (status) {
    case "active":
      return "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950 dark:text-emerald-300 dark:border-emerald-800";
    case "completed":
      return "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700";
    case "cancelled":
      return "bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950 dark:text-rose-300 dark:border-rose-800";
    case "scheduled":
      return "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950 dark:text-amber-300 dark:border-amber-800";
    default:
      return "bg-slate-50 text-slate-500";
  }
};

const getLocationName = (locationId: string | null) => {
  if (!locationId) return "N/A";
  return locationId; // Raw UUID for now
};

export default function EmployeeAssignmentsTab({employeeId, formatDate }: EmployeeAssignmentsTabProps) {
  const { data: assignments = [], isLoading } = useEmployeeAssignments(employeeId);
  const deleteAssignment = useDeleteEmployeeAssignment();
  const [editingAssignment, setEditingAssignment] = useState<EmployeeAssignment | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="space-y-4 animate-pulse pt-4">
        <div className="h-20 bg-slate-100 dark:bg-slate-800 rounded-2xl w-full"></div>
        <div className="h-20 bg-slate-100 dark:bg-slate-800 rounded-2xl w-full"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6 pt-4">
      <div className="flex justify-between items-center px-2">
        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 opacity-70">
          <Briefcase className="text-primary-500" size={14} /> Phái cử / Chuyển công tác
        </span>
        <PermissionGate permission="employees:update">
          <Button
            className="flex items-center gap-1.5 text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:bg-primary-50 dark:hover:bg-primary-900/50 py-1.5 px-3 rounded-full transition-colors h-auto shadow-none border-none"
            onClick={() => {
              setEditingAssignment(null);
              setIsModalOpen(true);
            }}
            type="button"
            variant="ghost"
          >
            <Plus size={14} /> Tạo phái cử
          </Button>
        </PermissionGate>
      </div>

      <div className="space-y-4 px-2">
        {assignments.length === 0 ? (
          <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] flex flex-col items-center justify-center text-center shadow-inner group hover:bg-white dark:hover:bg-slate-900 transition-all">
            <Briefcase className="w-12 h-12 text-slate-300 dark:text-slate-600 mb-3 group-hover:text-primary-400 transition-colors" />
            <div className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">Không có lịch sử phái cử</div>
          </div>
        ) : (
          <div className="relative pl-12 space-y-8 before:absolute before:left-[11px] before:top-4 before:bottom-4 before:w-px before:bg-slate-100 dark:before:bg-slate-700">
            {assignments.map((assignment: EmployeeAssignment, index: number) => (
              <div className="relative animate-in slide-in-from-left duration-500" key={assignment.id}>
                <div
                  className={cn(
                    "absolute -left-[3.15rem] top-1 w-6 h-6 rounded-full border-4 border-white dark:border-slate-900 shadow-xl ring-4 z-10",
                    index === 0 && assignment.status === "active"
                      ? "bg-emerald-50 dark:bg-emerald-950 ring-emerald-50 dark:ring-emerald-900/30 animate-pulse"
                      : "bg-primary-600 ring-primary-50 dark:ring-primary-900/30",
                  )}
                ></div>
                <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm hover:shadow-xl hover:shadow-primary-100/10 transition-all group overflow-hidden relative">
                  <div className="flex justify-between items-start mb-4">
                    <div>
                      <div className="flex items-center gap-3 mb-1.5">
                        <div className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">
                          {getLocationName(assignment.host_location_id)}
                        </div>
                        <span
                          className={cn(
                            "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 rounded-full border shadow-sm",
                            getStatusColor(assignment.status),
                          )}
                        >
                          {assignment.status === "active" ? t("employees.table.active").toUpperCase() : assignment.status.toUpperCase()}
                        </span>
                      </div>
                      {!!assignment.reason && (
                        <div className="text-[10px] font-mono text-slate-500 dark:text-slate-400">
                          {assignment.reason}
                        </div>
                      )}
                    </div>
                    <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity relative z-20">
                      <Button
                        className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => {
                          setEditingAssignment(assignment);
                          setIsModalOpen(true);
                        }}
                        size="icon"
                        type="button"
                        variant="ghost"
                      >
                        <Pencil size={14} />
                      </Button>
                      <Button
                        className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none"
                        onClick={() => {
                          deleteAssignment.mutate({employeeId, assignmentId: assignment.id });
                        }}
                        size="icon"
                        type="button"
                        variant="ghost"
                      >
                        <Trash2 size={14} />
                      </Button>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4 mt-6 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl relative z-10">
                    <div>
                      <div className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1 flex items-center gap-1.5">{t("employees.assignments.start_date", "Ngày bắt đầu")}</div>
                      <div className="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">
                        {formatDate(assignment.start_date)}
                      </div>
                    </div>
                    <div>
                      <div className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1 flex items-center gap-1.5">{t("employees.assignments.end_date", "Ngày kết thúc")}</div>
                      <div className="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">
                        {assignment.end_date ? formatDate(assignment.end_date) : t("employees.assignments.no_end_date", "Vô thời hạn")}
                      </div>
                    </div>
                  </div>
                  <Briefcase className="absolute -right-6 -bottom-6 w-24 h-24 text-slate-100 dark:text-slate-800 opacity-50 -rotate-12 group-hover:rotate-0 transition-transform duration-700 z-0 pointer-events-none" />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <EmployeeAssignmentFormModal editingAssignment={editingAssignment}
      employeeId={employeeId}
      isOpen={isModalOpen}
      onClose={() => {
        setIsModalOpen(false);
      }}  />
    </div>
  );
}

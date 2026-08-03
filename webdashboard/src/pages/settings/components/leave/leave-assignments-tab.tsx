import { Info, UserPlus, Trash2, Pencil, Users, Calendar } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Employee, Department } from "@/lib/api/types";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import ResponsiveTable from "@/components/ui/responsive-table";
import { toast } from "@/components/ui/toast-imperative";
import { Typography } from "@/components/ui/typography";
import { logError } from "@/lib/logger";
import { cn } from "@/lib/utils";

import type { LeaveAssignment } from "../../hooks/use-leave-settings";

interface LeaveAssignmentsTabProps {
  assignments: LeaveAssignment[];
  departments: Department[];
  employees: Employee[];
  isDeleting: boolean;
  isLoading: boolean;
  onAssignClick: () => void;
  onDelete: (id: string) => Promise<void>;
  onEditClick: (assignment: LeaveAssignment) => void;
}

  // eslint-disable-next-line max-lines-per-function
export function LeaveAssignmentsTab({
  assignments,
  isLoading,
  onAssignClick,
  onEditClick,
  employees,
  departments,
  onDelete,
  isDeleting,
}: LeaveAssignmentsTabProps) {
  const { t } = useTranslation();
  const [deletingAssignmentId, setDeletingAssignmentId] = useState<string | null>(null);

  const handleDeleteConfirm = async () => {
    if (deletingAssignmentId) {
      try {
        await onDelete(deletingAssignmentId);
        toast.success(t("settings.leave.messages.delete_assignment_success"));
      } catch (e: unknown) {
        logError(e, { component: 'LeaveAssignmentsTab.delete' });
        toast.error(t("settings.leave.messages.delete_assignment_error"));
      } finally {
        setDeletingAssignmentId(null);
      }
    }
  };


  return (
    <div>
      <div className="mb-8 border-b border-slate-50 dark:border-slate-800 pb-6 flex justify-between items-end">
        <div>
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">
            {t("settings.leave.tabs.assignments")}
          </Typography>
          <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 mt-1 uppercase tracking-tight" variant="p">
            {t("settings.leave.tabs.assignments_desc")}
          </Typography>
        </div>
        <Button className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 shadow-lg shadow-primary-100"
          data-testid="assign-policy-btn"
          onClick={() => { onAssignClick(); }}
          variant="ghost"
        >
          <UserPlus className="w-4 h-4" />
          {t("settings.leave.actions.assign")}
        </Button>
      </div>

      <div className="bg-amber-50 dark:bg-amber-950 border border-amber-100 dark:border-amber-800 p-8 rounded-[2.5rem] flex items-start gap-6">
        <Info className="text-amber-600 dark:text-amber-300 w-6 h-6 mt-1 shrink-0" />
        <div className="space-y-2">
          <Typography className="font-bold text-amber-900 uppercase tracking-widest text-xs" variant="p">{t("common.important_note")}</Typography>
          <Typography className="text-sm text-amber-800 leading-relaxed" variant="p">
            {t("settings.leave.messages.assignment_note")}
          </Typography>
        </div>
      </div>

      <div className="mt-8">
      <ResponsiveTable<LeaveAssignment>
        data={assignments}
        emptyState={{
          icon: <Users className="h-8 w-8" />,
          title: t("settings.leave.messages.no_assignments"),
        }}
        getCardKey={(a) => a.id}
        isLoading={isLoading}
        renderCard={(a) => {
          const getAssigneeName = () => {
            if (a.assignee_type === "workspace") {
              return t("leave_requests.policy.assignee_types.workspace");
            }
            if (a.assignee_type === "department") {
              const dept = departments.find((d) => d.id === a.assignee_id);
              return dept ? dept.name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
            }
            if (a.assignee_type === "employee") {
              const emp = employees.find((e) => e.id === a.assignee_id);
              return emp ? emp.full_name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
            }
            return t("common.unknown");
          };

          return (
            <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
              {/* Header: Policy name + Leave type */}
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-bold text-slate-700 dark:text-slate-200 tracking-tight">
                    {a.policy_name}
                  </div>
                  <div className="mt-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {a.leave_type_name}
                  </div>
                </div>
                <span className={cn("shrink-0 px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border shadow-sm",
                  a.assignee_type === "workspace" ? "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100" :
                  a.assignee_type === "department" ? "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800" :
                  "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                )}>
                  {a.assignee_type}
                </span>
              </div>

              {/* Detail grid */}
              <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
                <div>
                  <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("settings.leave.table.assignee")}</div>
                  <div className="mt-1 flex items-center gap-1.5">
                    <Users className="h-3 w-3 text-slate-400" />
                    <span className="text-xs font-bold text-slate-600 dark:text-slate-300 truncate max-w-[140px]">
                      {a.assignee_type !== "workspace" ? getAssigneeName() : getAssigneeName()}
                    </span>
                  </div>
                </div>
                <div>
                  <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("settings.leave.table.effective_date")}</div>
                  <div className="mt-1 flex items-center gap-1.5">
                    <Calendar className="h-3 w-3 text-slate-400" />
                    <span className="text-[10px] font-mono font-bold text-slate-600 dark:text-slate-300">
                      {format(new Date(a.effective_from), "MMM dd, yyyy")}
                      <span className="text-slate-300 dark:text-slate-600 mx-1">→</span>
                      {a.effective_to ? format(new Date(a.effective_to), "MMM dd, yyyy") : <span className="text-slate-400 dark:text-slate-500 italic">{t("common.forever")}</span>}
                    </span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="mt-4 flex items-center justify-between border-t border-slate-100 pt-4 dark:border-slate-800">
                <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 uppercase">
                  {format(new Date(a.created_at), "MMM dd HH:mm")}
                </span>
                <div className="flex items-center gap-1">
                  <Button
                    aria-label={t("common.edit")}
                    className="text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/50 px-2 py-1 h-8 rounded-lg"
                    disabled={isDeleting}
                    onClick={() => { onEditClick(a); }}
                    size="sm"
                    variant="ghost"
                  >
                    <Pencil className="w-4 h-4" />
                  </Button>
                  <Button
                    aria-label={t("settings.leave.actions.delete")}
                    className="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/20 px-2 py-1 h-8 rounded-lg"
                    disabled={isDeleting}
                    onClick={() => { setDeletingAssignmentId(a.id); }}
                    size="sm"
                    variant="ghost"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </div>
          );
        }}
        renderDesktopTable={() => (
          <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm mt-8 animate-in slide-in-from-bottom-4 duration-700">
            <table className="w-full text-left border-collapse">
              <thead className="bg-slate-50 dark:bg-slate-800 border-b border-slate-100 dark:border-slate-800">
                <tr>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.policy")}</th>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.type")}</th>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.assignee")}</th>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("settings.leave.table.effective_date")}</th>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("settings.leave.table.created_at")}</th>
                  <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("settings.leave.table.actions")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                {assignments.map((a) => {
                  const getAssigneeName = () => {
                    if (a.assignee_type === "workspace") {
                      return t("leave_requests.policy.assignee_types.workspace");
                    }
                    if (a.assignee_type === "department") {
                      const dept = departments.find((d) => d.id === a.assignee_id);
                      return dept ? dept.name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
                    }
                    if (a.assignee_type === "employee") {
                      const emp = employees.find((e) => e.id === a.assignee_id);
                      return emp ? emp.full_name : (a.assignee_id ? `${a.assignee_id.slice(0, 8)}…` : t("common.unknown"));
                    }
                    return t("common.unknown");
                  };

                  return (
                    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={a.id}>
                      <td className="px-6 py-5 font-bold text-slate-700 dark:text-slate-200 text-xs tracking-tight">{a.policy_name}</td>
                      <td className="px-6 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{a.leave_type_name}</td>
                      <td className="px-6 py-5">
                        <div className="flex items-center gap-3">
                          <span className={cn("px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border shadow-sm shrink-0",
                            a.assignee_type === "workspace" ? "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100" :
                            a.assignee_type === "department" ? "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800" :
                            "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                          )}>
                            {a.assignee_type}
                          </span>
                          {a.assignee_type !== "workspace" && (
                            <span className="font-bold text-slate-700 dark:text-slate-200 text-xs truncate max-w-[180px]">
                              {getAssigneeName()}
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="px-6 py-5 text-xs font-mono text-slate-600 dark:text-slate-300 font-bold">
                         {format(new Date(a.effective_from), "MMM dd, yyyy")}
                         <span className="text-slate-300 dark:text-slate-600 mx-2">→</span>
                         {a.effective_to ? format(new Date(a.effective_to), "MMM dd, yyyy") : <span className="text-slate-400 dark:text-slate-500 italic">{t("common.forever")}</span>}
                      </td>
                      <td className="px-6 py-5 text-[10px] font-mono text-slate-400 dark:text-slate-500 text-right uppercase">
                        {format(new Date(a.created_at), "MMM dd HH:mm")}
                      </td>
                      <td className="px-6 py-5 text-right flex items-center justify-end gap-2">
                        <Button
                          aria-label={t("common.edit")}
                          className="text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/50 px-2 py-1 h-8 rounded-lg transition-colors inline-flex items-center justify-center"
                          disabled={isDeleting}
                          onClick={() => { onEditClick(a); }}
                          size="sm"
                          variant="ghost"
                        >
                          <Pencil className="w-4 h-4" />
                        </Button>
                        <Button
                          aria-label={t("settings.leave.actions.delete")}
                          className="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/20 px-2 py-1 h-8 rounded-lg transition-colors inline-flex items-center justify-center"
                          disabled={isDeleting}
                          onClick={() => { setDeletingAssignmentId(a.id); }}
                          size="sm"
                          variant="ghost"
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      />
      </div>

      <ConfirmDialog
        confirmLabel={t("settings.leave.actions.delete")}
        isLoading={isDeleting}
        isOpen={deletingAssignmentId !== null}
        message={t("settings.leave.dialogs.delete_assignment_message")}
        onClose={() => { setDeletingAssignmentId(null); }}
        onConfirm={handleDeleteConfirm}
        title={t("settings.leave.dialogs.delete_assignment_title")}
        variant="danger"
      />
    </div>
  );
}

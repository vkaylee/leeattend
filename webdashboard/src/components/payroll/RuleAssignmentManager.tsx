import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Users, Building, Building2, Plus, Trash2 } from "lucide-react";
import { apiFetch } from "@/lib/api/helpers";
import { API_BASE_URL } from "@/lib/api/config";
import { useToast } from "@/components/ui/Toast";
import { useTranslation } from "react-i18next";
import { PayrollRuleAssignment } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  wsId: string;
  ruleId: string;
}

export default function RuleAssignmentManager({ wsId, ruleId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: assignments, isLoading } = useQuery({
    queryKey: ["payroll-rule-assignments", ruleId],
    queryFn: async () => {
      const res = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`);
      if (!res.ok) throw new Error("Failed to fetch assignments");
      return res.json() as Promise<PayrollRuleAssignment[]>;
    },
    enabled: !!ruleId,
  });

  const [newAssigneeType, setNewAssigneeType] = useState<"workspace" | "department" | "employee">("workspace");
  const [newAssigneeId, setNewAssigneeId] = useState<string>("");

  const createMutation = useMutation({
    mutationFn: async (data: Partial<PayrollRuleAssignment>) => {
      const res = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/payroll-rules/${ruleId}/assignments`, {
        method: "POST",
        body: JSON.stringify(data),
      });
      if (!res.ok) throw new Error("Failed to create assignment");
      return res.json();
    },
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-rule-assignments", ruleId] });
      setNewAssigneeId("");
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({
      assignee_type: newAssigneeType,
      assignee_id: newAssigneeType === "workspace" ? wsId : newAssigneeId,
      effective_from: new Date().toISOString().split("T")[0],
    });
  };

  return (
    <div className="space-y-6 mt-4">
      <div className="bg-slate-50 dark:bg-slate-800/50 rounded-2xl p-6 border border-slate-100 dark:border-slate-800">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-8 h-8 rounded-lg bg-teal-100 dark:bg-teal-900/30 text-teal-600 flex items-center justify-center">
            <Users size={16} />
          </div>
          <div>
            <h3 className="font-bold text-sm uppercase tracking-widest text-slate-700 dark:text-slate-200">Rule Assignments</h3>
            <p className="text-[10px] text-slate-500 uppercase tracking-widest mt-1">Who does this rule apply to?</p>
          </div>
        </div>

        {isLoading ? (
          <div className="p-10 text-center text-slate-500">Loading assignments...</div>
        ) : (
          <div className="space-y-2">
            {assignments?.map((assignment) => (
              <div key={assignment.id} className="flex items-center justify-between p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm">
                <div className="flex items-center gap-4">
                  <div className={cn("w-10 h-10 rounded-lg flex items-center justify-center", 
                    assignment.assignee_type === "workspace" ? "bg-indigo-50 text-indigo-600" :
                    assignment.assignee_type === "department" ? "bg-purple-50 text-purple-600" :
                    "bg-blue-50 text-blue-600"
                  )}>
                    {assignment.assignee_type === "workspace" && <Building2 size={20} />}
                    {assignment.assignee_type === "department" && <Building size={20} />}
                    {assignment.assignee_type === "employee" && <Users size={20} />}
                  </div>
                  <div>
                    <p className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight">
                      {assignment.assignee_type} Level
                    </p>
                    <p className="text-[10px] text-slate-500 uppercase tracking-widest mt-1">ID: {assignment.assignee_id.slice(0, 8)}...</p>
                  </div>
                </div>
                <button className="text-slate-400 hover:text-rose-500 transition-colors"><Trash2 size={18} /></button>
              </div>
            ))}

            <form onSubmit={handleSubmit} className="mt-6 p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-700 shadow-inner flex items-end gap-4">
              <div className="flex-1 space-y-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Assignee Type</label>
                <select className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold uppercase tracking-tight" value={newAssigneeType} onChange={e => setNewAssigneeType(e.target.value as any)}>
                  <option value="workspace">Entire Workspace</option>
                  <option value="department">Specific Department</option>
                  <option value="employee">Specific Employee</option>
                </select>
              </div>
              
              {newAssigneeType !== "workspace" && (
                <div className="flex-[2] space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Assignee ID (UUID)</label>
                  <input type="text" required placeholder="Enter UUID" className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold" value={newAssigneeId} onChange={e => setNewAssigneeId(e.target.value)} />
                </div>
              )}
              
              <button type="submit" disabled={createMutation.isPending} className="bg-teal-600 text-white px-6 py-2 rounded-lg font-bold text-xs uppercase tracking-widest hover:bg-teal-700 transition-colors disabled:opacity-50 h-[38px]">
                Assign
              </button>
            </form>
          </div>
        )}
      </div>
    </div>
  );
}

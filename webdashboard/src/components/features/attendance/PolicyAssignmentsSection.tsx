import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { 
  fetchPolicyAssignments, 
  upsertPolicyAssignment, 
  deletePolicyAssignment, 
  fetchDepartments, 
  fetchEmployees,
  PolicyAssignmentResponse,
  Department,
  Employee
} from "@/lib/api";
import { 
  Building2, 
  User, 
  Plus, 
  Settings2, 
  Clock, 
  ShieldAlert, 
  Calculator,
  Zap,
  MoreVertical,
  Trash2,
  ChevronRight,
  Filter
} from "lucide-react";
import { cn } from "@/lib/utils";
import Button from "@/components/ui/Button";
import { LABEL_BASE } from "@/components/ui/FormField";
import { Combobox } from "@/components/ui/Combobox";
import { useToast } from "@/components/ui/Toast";
import Sheet from "@/components/ui/Sheet";
import { UpsertPolicyAssignmentRequest } from "@/lib/api";

// We'll build a slide-over later, but for now let's structure the table.

export function PolicyAssignmentsSection({ wsId }: { wsId: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: assignments = [], isLoading: isLoadingAssignments } = useQuery({
    queryKey: ["attendance-policy-assignments", wsId],
    queryFn: () => fetchPolicyAssignments(wsId),
  });

  const { data: departments = [] } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
  });

  const { data: employeesResponse } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId, 1, 1000),
  });

  const employees = employeesResponse?.data || [];

  const [activeFilter, setActiveFilter] = useState<"ALL" | "GROUP" | "USER">("ALL");

  const filteredAssignments = assignments.filter(a => activeFilter === "ALL" || a.target_type === activeFilter);

  // Helper to resolve name
  const getTargetName = (type: string, id: string) => {
    if (type === "GROUP") {
      const dept = departments.find(d => d.id === id);
      return dept ? dept.name : "Unknown Department";
    } else {
      const emp = employees.find((e: Employee) => e.id === id);
      return emp ? emp.full_name : "Unknown Employee";
    }
  };

  const workUnitLabel = (v: number) => {
    if (v === 1) return "1 min (exact)";
    return `${v} min`;
  };

  const roundingLabel = (v: string) => {
    const map: Record<string, string> = { none: "No rounding", floor: "Round down", ceil: "Round up", round: "Standard rounding" };
    return map[v] || v;
  };

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deletePolicyAssignment(wsId, id),
    onSuccess: () => {
      toast.success("Policy override removed");
      queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments", wsId] });
    }
  });

  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [targetType, setTargetType] = useState<"USER" | "GROUP">("GROUP");
  const [targetId, setTargetId] = useState("");
  const [workUnitMinutes, setWorkUnitMinutes] = useState(1);
  const [roundingPolicy, setRoundingPolicy] = useState("none");
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<"allow" | "block" | "clamp">("allow");

  const upsertMutation = useMutation({
    mutationFn: (data: UpsertPolicyAssignmentRequest) => upsertPolicyAssignment(wsId, data),
    onSuccess: () => {
      toast.success("Policy override saved");
      queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments", wsId] });
      setIsDrawerOpen(false);
      setTargetId("");
    }
  });

  const handleSave = () => {
    if (!targetId) {
      toast.error("Please select a department or employee");
      return;
    }
    upsertMutation.mutate({
      target_type: targetType,
      target_id: targetId,
      work_unit_minutes: workUnitMinutes,
      rounding_policy: roundingPolicy,
      early_arrival_strategy: earlyArrivalStrategy,
    });
  };

  const targetOptions = targetType === "GROUP"
    ? departments.map(d => ({ value: d.id, label: d.name, icon: <Building2 className="w-4 h-4 text-indigo-500" /> }))
    : employees.map((e: Employee) => ({ value: e.id, label: e.full_name, description: e.employee_id_number, icon: <User className="w-4 h-4 text-teal-500" /> }));


  if (isLoadingAssignments) return <div className="h-40 animate-pulse bg-slate-100 dark:bg-slate-800 rounded-[2.5rem]"></div>;

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden mt-8">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">Overrides & Exceptions</h2>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">Configure specific rules for departments or individuals.</p>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-xl">
              <button onClick={() => setActiveFilter("ALL")} className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "ALL" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")}>ALL</button>
              <button onClick={() => setActiveFilter("GROUP")} className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "GROUP" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")}>DEPTS</button>
              <button onClick={() => setActiveFilter("USER")} className={cn("px-4 py-2 rounded-lg text-xs font-bold tracking-widest uppercase transition-all", activeFilter === "USER" ? "bg-white dark:bg-slate-900 shadow-sm text-slate-800 dark:text-slate-200" : "text-slate-500 hover:text-slate-700")}>USERS</button>
            </div>
            <Button icon={<Plus size={18} />} onClick={() => setIsDrawerOpen(true)}>ADD OVERRIDE</Button>
          </div>
        </div>
      </div>

      <div className="p-10">
        {filteredAssignments.length === 0 ? (
          <div className="text-center py-12 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-800/30">
            <Filter className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-4" />
            <h3 className="text-lg font-bold text-slate-600 dark:text-slate-300">No active overrides</h3>
            <p className="text-sm text-slate-500 mt-1">Everyone is following the workspace default policy.</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredAssignments.map(assignment => (
              <div key={assignment.id} className="group relative border border-slate-100 dark:border-slate-800 rounded-2xl p-6 bg-slate-50/50 dark:bg-slate-800/20 hover:bg-white dark:hover:bg-slate-800 transition-all flex items-center justify-between">
                <div className="flex items-center gap-5">
                  <div className={cn("w-12 h-12 rounded-xl flex items-center justify-center shadow-sm border", assignment.target_type === "GROUP" ? "bg-indigo-50 dark:bg-indigo-950/30 border-indigo-100 dark:border-indigo-800 text-indigo-600 dark:text-indigo-400" : "bg-teal-50 dark:bg-teal-950/30 border-teal-100 dark:border-teal-800 text-teal-600 dark:text-teal-400")}>
                    {assignment.target_type === "GROUP" ? <Building2 size={24} /> : <User size={24} />}
                  </div>
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <span className={cn("px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest rounded-md", assignment.target_type === "GROUP" ? "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300" : "bg-teal-100 text-teal-700 dark:bg-teal-900/50 dark:text-teal-300")}>
                        {assignment.target_type === "GROUP" ? "DEPARTMENT" : "EMPLOYEE"}
                      </span>
                    </div>
                    <h4 className="text-lg font-bold text-slate-800 dark:text-slate-200">
                      {getTargetName(assignment.target_type, assignment.target_id)}
                    </h4>
                  </div>
                </div>

                <div className="flex items-center gap-8 px-6">
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">Work Unit</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{workUnitLabel(assignment.rules.work_unit_minutes)}</p>
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">Rounding</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300">{roundingLabel(assignment.rules.rounding_policy)}</p>
                  </div>
                  <div>
                    <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-1">Early Arrival</p>
                    <p className="text-sm font-bold text-slate-700 dark:text-slate-300 capitalize">{assignment.rules.early_arrival_strategy}</p>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <button onClick={() => deleteMutation.mutate(assignment.id)} className="w-10 h-10 rounded-xl flex items-center justify-center text-slate-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/30 transition-colors">
                    <Trash2 size={18} />
                  </button>
                  <button className="w-10 h-10 rounded-xl flex items-center justify-center text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                    <ChevronRight size={20} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <Sheet
        isOpen={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
        title="Add Policy Override"
        subtitle="Configure specific rules for a target"
      >
        <div className="space-y-8">
          <div className="space-y-4">
            <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest">1. Select Target</h3>
            
            <div className="grid grid-cols-2 gap-3 mb-4">
              <button
                className={cn("p-4 rounded-2xl border transition-all flex flex-col items-center gap-2", targetType === "GROUP" ? "bg-indigo-50 dark:bg-indigo-950/30 border-indigo-200 dark:border-indigo-800 text-indigo-700 dark:text-indigo-400 shadow-inner" : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-500 hover:border-slate-300 dark:hover:border-slate-700")}
                onClick={() => { setTargetType("GROUP"); setTargetId(""); }}
              >
                <Building2 size={24} />
                <span className="text-xs font-bold uppercase tracking-widest">Department</span>
              </button>
              <button
                className={cn("p-4 rounded-2xl border transition-all flex flex-col items-center gap-2", targetType === "USER" ? "bg-teal-50 dark:bg-teal-950/30 border-teal-200 dark:border-teal-800 text-teal-700 dark:text-teal-400 shadow-inner" : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-500 hover:border-slate-300 dark:hover:border-slate-700")}
                onClick={() => { setTargetType("USER"); setTargetId(""); }}
              >
                <User size={24} />
                <span className="text-xs font-bold uppercase tracking-widest">Employee</span>
              </button>
            </div>

            <div>
              <label className={LABEL_BASE}>Select {targetType === "GROUP" ? "Department" : "Employee"}</label>
              <Combobox
                options={targetOptions}
                value={targetId}
                onChange={setTargetId}
                placeholder={targetType === "GROUP" ? "Search department..." : "Search employee..."}
              />
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest">2. Configure Rules</h3>
            
            <div className="space-y-5 bg-slate-50/50 dark:bg-slate-800/30 p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800">
              <div>
                <label className={LABEL_BASE}>Work Unit</label>
                <select 
                  className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm font-bold text-slate-700 dark:text-slate-300 outline-none focus:ring-2 focus:ring-primary-500/20"
                  value={workUnitMinutes}
                  onChange={e => setWorkUnitMinutes(Number(e.target.value))}
                >
                  <option value={1}>1 Minute (Exact)</option>
                  <option value={15}>15 Minutes</option>
                  <option value={30}>30 Minutes</option>
                  <option value={60}>60 Minutes</option>
                </select>
              </div>

              <div>
                <label className={LABEL_BASE}>Rounding Policy</label>
                <select 
                  className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm font-bold text-slate-700 dark:text-slate-300 outline-none focus:ring-2 focus:ring-primary-500/20"
                  value={roundingPolicy}
                  onChange={e => setRoundingPolicy(e.target.value)}
                >
                  <option value="none">No Rounding</option>
                  <option value="floor">Round Down (Floor)</option>
                  <option value="ceil">Round Up (Ceil)</option>
                  <option value="round">Standard Rounding</option>
                </select>
              </div>

              <div>
                <label className={LABEL_BASE}>Early Arrival Strategy</label>
                <select 
                  className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm font-bold text-slate-700 dark:text-slate-300 outline-none focus:ring-2 focus:ring-primary-500/20"
                  value={earlyArrivalStrategy}
                  onChange={e => setEarlyArrivalStrategy(e.target.value as "allow" | "block" | "clamp")}
                >
                  <option value="allow">Allow Always</option>
                  <option value="clamp">Clamp to Grace Period</option>
                  <option value="block">Block Early Punch-in</option>
                </select>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-slate-100 dark:border-slate-800">
            <Button 
              className="w-full" 
              onClick={handleSave} 
              loading={upsertMutation.isPending}
            >
              SAVE OVERRIDE
            </Button>
          </div>
        </div>
      </Sheet>
    </div>
  );
}

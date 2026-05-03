

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2, Database, Info, Plus, X } from "lucide-react";
import React from "react";

import type { SalaryBaseGroup, SalaryBaseGroupMember, SalaryComponentDefinition } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { 
  fetchSalaryBaseGroups, 
  fetchSalaryComponents,
  assignComponentToSalaryBase,
  unassignComponentFromSalaryBase
} from "@/lib/api/payroll";

function BaseGroupCard({
  group,
  members,
  components,
  isPending,
  onAssign,
  onUnassign
}: {
  components: SalaryComponentDefinition[];
  group: SalaryBaseGroup;
  members: SalaryBaseGroupMember[];
  isPending: boolean;
  onAssign: (baseId: string, componentId: string) => void;
  onUnassign: (baseId: string, componentId: string) => void;
}) {
  const groupMembers = members.filter(m => m.base_group_id === group.id);
  const memberIds = new Set(groupMembers.map(m => m.component_definition_id));

  return (
    <div className="bg-white dark:bg-slate-800/50 rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm hover:shadow-md transition-shadow">
      <div className="p-5 border-b border-slate-100 dark:border-slate-700/50 bg-slate-50 dark:bg-slate-800/80">
        <div className="flex justify-between items-start mb-2">
          <h3 className="font-semibold text-slate-800 dark:text-slate-200">{group.name}</h3>
          {group.is_system ? <span className="bg-primary-50 text-primary-600 text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full dark:bg-primary-900/30 dark:text-primary-400 border border-primary-100 dark:border-primary-800/50">System</span> : null}
        </div>
        <p className="text-xs text-slate-500 line-clamp-2">{group.description ?? "No description provided."}</p>
        <div className="mt-3 text-xs font-mono text-slate-400 bg-slate-100 dark:bg-slate-900 px-2 py-1 rounded inline-block">
          {group.slug}
        </div>
      </div>

      <div className="p-4">
        <h4 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-3">Included Components</h4>
        <div className="space-y-2 max-h-64 overflow-y-auto pr-2 custom-scrollbar">
          {components.length === 0 ? (
            <div className="text-center py-4 text-sm text-slate-400 flex flex-col items-center">
              <Info className="w-5 h-5 mb-1 opacity-50" />
              No components available
            </div>
          ) : (
            components.map(comp => {
              const isAssigned = memberIds.has(comp.id);
              
              return (
                <div 
                  className={`flex items-center justify-between p-2.5 rounded-xl border transition-colors ${
                    isAssigned 
                      ? 'border-primary-200 bg-primary-50/50 dark:border-primary-900/50 dark:bg-primary-900/20' 
                      : 'border-slate-100 bg-white hover:border-slate-200 dark:border-slate-700/50 dark:bg-slate-800 dark:hover:border-slate-600'
                  }`} 
                  key={comp.id}
                >
                  <div className="flex-1 min-w-0 pr-3">
                    <p className={`text-sm font-medium truncate ${isAssigned ? 'text-primary-700 dark:text-primary-300' : 'text-slate-600 dark:text-slate-300'}`}>
                      {comp.name}
                    </p>
                    <p className="text-xs text-slate-400 font-mono mt-0.5">{comp.code}</p>
                  </div>
                  
                  <button
                    className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center transition-colors ${
                      isAssigned 
                        ? 'bg-rose-50 text-rose-500 hover:bg-rose-100 dark:bg-rose-500/10 dark:hover:bg-rose-500/20' 
                        : 'bg-slate-100 text-slate-400 hover:bg-primary-50 hover:text-primary-600 dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-primary-500/20 dark:hover:text-primary-400'
                    } disabled:opacity-50 disabled:cursor-not-allowed`}
                    disabled={isPending || (group.slug === 'gross')}
                    onClick={() => {
                      if (isAssigned) {
                        onUnassign(group.id, comp.id);
                      } else {
                        onAssign(group.id, comp.id);
                      }
                    }}
                    title={group.slug === 'gross' ? 'Gross base automatically includes all components' : isAssigned ? 'Unassign Component' : 'Assign Component'}
                  >
                    {isAssigned ? <X className="w-4 h-4" /> : <Plus className="w-4 h-4" />}
                  </button>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}

export default function SalaryBaseGroupManager({ wsId }: { wsId: string }) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: salaryBasesResponse, isLoading: loadingGroups } = useQuery({
    queryKey: ["salary-base-groups", wsId],
    queryFn: () => fetchSalaryBaseGroups(wsId),
  });

  const { data: components, isLoading: loadingComponents } = useQuery({
    queryKey: ["salary-components", wsId],
    queryFn: () => fetchSalaryComponents(wsId),
  });

  const groups = salaryBasesResponse?.groups;
  const members = salaryBasesResponse?.members;

  const assignMutation = useMutation({
    mutationFn: ({ baseId, componentId }: { baseId: string, componentId: string }) => 
      assignComponentToSalaryBase(wsId, baseId, componentId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["salary-base-members", wsId] });
      toast.success("Component assigned to base successfully");
    },
    onError: () => {
      toast.error("Failed to assign component");
    }
  });

  const unassignMutation = useMutation({
    mutationFn: ({ baseId, componentId }: { baseId: string, componentId: string }) => 
      unassignComponentFromSalaryBase(wsId, baseId, componentId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["salary-base-members", wsId] });
      toast.success("Component unassigned from base successfully");
    },
    onError: () => {
      toast.error("Failed to unassign component");
    }
  });

  if (loadingGroups || loadingComponents) {
    return (
      <div className="flex items-center justify-center p-12 text-slate-400" data-testid="loader">
        <Loader2 className="animate-spin w-8 h-8" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <Database className="w-5 h-5 text-primary-500" />
            Salary Bases
          </h2>
          <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            Group your salary components into functional bases (e.g., Gross Salary, BHXH Base) to be used in Payroll Rules.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
        {groups?.map(group => (
          <BaseGroupCard
            components={components ?? []}
            group={group}
            isPending={assignMutation.isPending || unassignMutation.isPending}
            key={group.id}
            members={members ?? []}
            onAssign={(baseId, componentId) => { assignMutation.mutate({ baseId, componentId }); }}
            onUnassign={(baseId, componentId) => { unassignMutation.mutate({ baseId, componentId }); }}
          />
        ))}
      </div>
    </div>
  );
}

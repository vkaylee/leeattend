import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2, Database, Info, Plus, X } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { SalaryBaseGroup, SalaryBaseGroupMember, SalaryComponentDefinition } from "@/lib/api";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { 
  fetchSalaryBaseGroups, 
  fetchSalaryComponents,
  assignComponentToSalaryBase,
  unassignComponentFromSalaryBase,
  createSalaryBaseGroup
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
  const { t } = useTranslation();
  const groupMembers = members.filter(m => m.base_group_id === group.id);
  const memberIds = new Set(groupMembers.map(m => m.component_definition_id));

  const displayName = group.is_system ? t(`payroll.system_bases.${group.slug}.name`, { defaultValue: group.name }) : group.name;
  const displayDesc = group.is_system ? t(`payroll.system_bases.${group.slug}.description`, { defaultValue: group.description ?? "No description provided." }) : (group.description ?? "No description provided.");

  return (
    <div className="bg-white dark:bg-slate-800/50 rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm hover:shadow-md transition-shadow">
      <div className="p-5 border-b border-slate-100 dark:border-slate-700/50 bg-slate-50 dark:bg-slate-800/80">
        <div className="flex justify-between items-start mb-2">
          <Typography className="font-semibold text-slate-800 dark:text-slate-200" variant="h3">{displayName}</Typography>
          {group.is_system ? <span className="bg-primary-50 text-primary-600 text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full dark:bg-primary-900/30 dark:text-primary-400 border border-primary-100 dark:border-primary-800/50">System</span> : null}
        </div>
        <Typography className="text-xs text-slate-500 line-clamp-2" variant="p">{displayDesc}</Typography>
        <div className="mt-3 text-xs font-mono text-slate-400 bg-slate-100 dark:bg-slate-900 px-2 py-1 rounded inline-block">
          {group.slug}
        </div>
      </div>

      <div className="p-4">
        <Typography className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-3" variant="h4">Included Components</Typography>
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
                    <Typography className={`text-sm font-medium truncate ${isAssigned ? 'text-primary-700 dark:text-primary-300' : 'text-slate-600 dark:text-slate-300'}`} variant="p">
                      {comp.name}
                    </Typography>
                    <Typography className="text-xs text-slate-400 font-mono mt-0.5" variant="p">{comp.code}</Typography>
                  </div>
                  
                  <Button
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
                  </Button>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}

function CreateBaseModal({
  isOpen,
  onClose,
  wsId,
}: {
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [description, setDescription] = useState("");

  const createMutation = useMutation({
    mutationFn: () => createSalaryBaseGroup(wsId, { name, slug, description }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["salary-base-groups", wsId] });
      toast.success("Salary base created successfully");
      onClose();
      setName("");
      setSlug("");
      setDescription("");
    },
    onError: () => {
      toast.error("Failed to create salary base");
    },
  });

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Create Salary Base">
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1" htmlFor="base-name">
            Base Name
          </label>
          <input
            className="w-full p-2 border rounded-lg dark:bg-slate-800 dark:border-slate-700 dark:text-slate-200"
            id="base-name"
            onChange={(e) => {
              setName(e.target.value);
              setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9]+/g, "_"));
            }}
            placeholder="e.g. Hazard Base"
            value={name}
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1" htmlFor="base-slug">
            Slug
          </label>
          <input
            className="w-full p-2 border rounded-lg dark:bg-slate-800 dark:border-slate-700 dark:text-slate-200"
            id="base-slug"
            onChange={(e) => { setSlug(e.target.value); }}
            placeholder="e.g. hazard_base"
            value={slug}
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1" htmlFor="base-description">
            Description
          </label>
          <textarea
            className="w-full p-2 border rounded-lg dark:bg-slate-800 dark:border-slate-700 dark:text-slate-200"
            id="base-description"
            onChange={(e) => { setDescription(e.target.value); }}
            placeholder="Description for this base group"
            rows={3}
            value={description}
          />
        </div>
        <div className="flex justify-end gap-3 pt-4">
          <Button
            className="px-4 py-2 text-slate-600 hover:bg-slate-100 rounded-lg dark:text-slate-400 dark:hover:bg-slate-800"
            onClick={onClose}
            type="button"
          >
            Cancel
          </Button>
          <Button
            className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 flex items-center gap-2"
            disabled={!name || !slug || createMutation.isPending}
            onClick={() => { createMutation.mutate(); }}
            type="button"
          >
            {createMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
            Create Base
          </Button>
        </div>
      </div>
    </Modal>
  );
}

export default function SalaryBaseGroupManager({ wsId }: { wsId: string }) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const [showCreateModal, setShowCreateModal] = useState(false);

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
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 flex items-center gap-2" variant="h2">
            <Database className="w-5 h-5 text-primary-500" />
            Salary Bases
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
            Group your salary components into functional bases (e.g., Gross Salary, BHXH Base) to be used in Payroll Rules.
          </Typography>
        </div>
        <Button
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white text-sm font-semibold rounded-xl shadow-sm transition-colors"
          onClick={() => { setShowCreateModal(true); }}
        >
          <Plus className="w-4 h-4" />
          Create Base
        </Button>
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

      <CreateBaseModal
        isOpen={showCreateModal}
        onClose={() => { setShowCreateModal(false); }}
        wsId={wsId}
      />
    </div>
  );
}

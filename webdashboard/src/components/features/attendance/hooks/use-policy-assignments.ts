import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { departmentKeys, employeeKeys } from "@/lib/query-keys";

import { useToast } from "@/components/ui/toast";
import {
  fetchPolicyAssignments,
  upsertPolicyAssignment,
  deletePolicyAssignment,
  fetchDepartments,
  fetchEmployees,
  Employee,
  UpsertPolicyAssignmentRequest,
  PolicyAssignmentResponse
} from "@/lib/api";

export type FilterType = "ALL" | "GROUP" | "USER";
export type StrategyType = "allow" | "block" | "clamp";

export function usePolicyAssignments() {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();

  // Form state
  const [activeFilter, setActiveFilter] = useState<FilterType>("ALL");
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [targetType, setTargetType] = useState<"USER" | "GROUP">("GROUP");
  const [targetId, setTargetId] = useState("");
  const [workUnitMinutes, setWorkUnitMinutes] = useState(1);
  const [roundingPolicy, setRoundingPolicy] = useState<"none" | "floor" | "round" | "ceil">("none");
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<StrategyType>("allow");
  const [validFrom, setValidFrom] = useState(() => new Date().toISOString().split("T")[0]);
  const [gracePeriodMinutes, setGracePeriodMinutes] = useState<number | undefined>(undefined);

  // Queries
  const { data: assignments = [], isLoading: isLoadingAssignments } = useQuery<PolicyAssignmentResponse[]>({ queryKey: ["attendance-policy-assignments"], queryFn: () => fetchPolicyAssignments() });

  const { data: departments = [] } = useOrganizationQuery({ queryKey: departmentKeys.all(),
  queryFn: fetchDepartments, });

  const { data: employeesResponse } = useOrganizationQuery({ queryKey: employeeKeys.lookup(),
  queryFn: () => fetchEmployees(1, 1000), });

  const employees = useMemo(() => employeesResponse?.data ?? [], [employeesResponse]);

  // Mutations
  const deleteMutation = useMutation({
    mutationFn: (id: string) => deletePolicyAssignment(id),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments"] });
    }
  });

  const upsertMutation = useMutation({
    mutationFn: (data: UpsertPolicyAssignmentRequest) => upsertPolicyAssignment(data),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.save_success"));
      void queryClient.invalidateQueries({ queryKey: ["attendance-policy-assignments"] });
    }
  });

  // Derived state & helpers
  const filteredAssignments = useMemo(() =>
    assignments.filter(a => activeFilter === "ALL" || a.target_type === activeFilter),
    [assignments, activeFilter]
  );

  const deptMap = useMemo(() => new Map(departments.map(d => [d.id, d.name])), [departments]);
  const empMap = useMemo(() => new Map(employees.map((e: Employee) => [e.id, e.full_name])), [employees]);

  const getTargetName = useCallback((type: string, id: string) =>
    type === "GROUP" ? deptMap.get(id) ?? "Unknown Department" : empMap.get(id) ?? "Unknown Employee",
    [deptMap, empMap]
  );

  const workUnitLabel = useCallback((v: number) =>
    t(`common.organization_settings.work_unit_options.${v}min`, { defaultValue: `${v} min` }),
    [t]
  );

  const roundingLabel = useCallback((v: string) =>
    t(`common.organization_settings.rounding_options.${v}`, { defaultValue: v }),
    [t]
  );

  const handleSave = () => {
    if (!targetId) {
      toast.error(t("attendance_policy_overrides.messages.validation_select_target"));
      return;
    }
    upsertMutation.mutate({
      target_type: targetType,
      target_id: targetId,
      work_unit_minutes: workUnitMinutes,
      rounding_policy: roundingPolicy,
      early_arrival_strategy: earlyArrivalStrategy,
      valid_from: validFrom,
      grace_period_minutes: gracePeriodMinutes ?? null
    }, {
      onSuccess: () => {
        setIsDrawerOpen(false);
        setTargetId("");
        setGracePeriodMinutes(undefined);
      }
    });
  };

  return {
    assignments,
    departments,
    employees,
    isLoadingAssignments,
    deleteMutation,
    upsertMutation,
    activeFilter,
    setActiveFilter,
    isDrawerOpen,
    setIsDrawerOpen,
    targetType,
    setTargetType,
    targetId,
    setTargetId,
    workUnitMinutes,
    setWorkUnitMinutes,
    roundingPolicy,
    setRoundingPolicy,
    earlyArrivalStrategy,
    setEarlyArrivalStrategy,
    validFrom,
    setValidFrom,
    gracePeriodMinutes,
    setGracePeriodMinutes,
    filteredAssignments,
    getTargetName,
    workUnitLabel,
    roundingLabel,
    handleSave
  };
}

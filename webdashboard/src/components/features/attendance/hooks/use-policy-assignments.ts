import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import {
  deletePolicyAssignment,
  Employee,
  fetchDepartments,
  fetchEmployees,
  fetchPolicyAssignments,
  upsertPolicyAssignment,
  UpsertPolicyAssignmentRequest,
} from "@/lib/api";
import { createQueryKey, departmentKeys, employeeKeys } from "@/lib/query-keys";

export type FilterType = "ALL" | "GROUP" | "USER";
export type StrategyType = "allow" | "block" | "clamp";

export interface PolicyAssignmentsOptions {
  limit?: number;
  onPageChange?: (page: number) => void;
  onTargetTypeChange?: (targetType?: Exclude<FilterType, "ALL">) => void;
  page?: number;
  q?: string;
  targetType?: Exclude<FilterType, "ALL">;
}

type RoundingPolicy = "none" | "floor" | "round" | "ceil";
type TargetType = "USER" | "GROUP";

function getTargetName({
  assignments,
  departments,
  employees,
  type,
  id,
}: {
  assignments: { target_id: string; target_name?: string | null; target_type: string }[];
  departments: Map<string, string>;
  employees: Map<string, string>;
  type: string;
  id: string;
}): string {
  const assignment = assignments.find((item) => item.target_type === type && item.target_id === id);
  if (assignment?.target_name) return assignment.target_name;
  return type === "GROUP" ? departments.get(id) ?? "Unknown Department" : employees.get(id) ?? "Unknown Employee";
}

function resolveServerTargetType(
  targetType: Exclude<FilterType, "ALL"> | undefined,
  activeFilter: FilterType,
): Exclude<FilterType, "ALL"> | undefined {
  if (targetType) return targetType;
  return activeFilter === "ALL" ? undefined : activeFilter;
}

function buildAssignmentRequest({
  targetType,
  targetId,
  workUnitMinutes,
  roundingPolicy,
  earlyArrivalStrategy,
  validFrom,
  gracePeriodMinutes,
}: {
  earlyArrivalStrategy: StrategyType;
  gracePeriodMinutes: number | undefined;
  roundingPolicy: RoundingPolicy;
  targetId: string;
  targetType: TargetType;
  validFrom: string;
  workUnitMinutes: number;
}): UpsertPolicyAssignmentRequest {
  return {
    target_type: targetType,
    target_id: targetId,
    work_unit_minutes: workUnitMinutes,
    rounding_policy: roundingPolicy,
    early_arrival_strategy: earlyArrivalStrategy,
    valid_from: validFrom,
    grace_period_minutes: gracePeriodMinutes ?? null,
  };
}

export function usePolicyAssignments(options: PolicyAssignmentsOptions = {}) {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { t } = useTranslation();
  const [localFilter, setLocalFilter] = useState<FilterType>("ALL");
  const activeFilter = options.targetType ?? localFilter;
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [targetType, setTargetType] = useState<TargetType>("GROUP");
  const [targetId, setTargetId] = useState("");
  const [workUnitMinutes, setWorkUnitMinutes] = useState(1);
  const [roundingPolicy, setRoundingPolicy] = useState<RoundingPolicy>("none");
  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState<StrategyType>("allow");
  const [validFrom, setValidFrom] = useState(() => new Date().toISOString().split("T")[0]);
  const [gracePeriodMinutes, setGracePeriodMinutes] = useState<number | undefined>(undefined);

  const page = options.page ?? 1;
  const limit = options.limit ?? 50;
  const serverTargetType = resolveServerTargetType(options.targetType, activeFilter);
  const assignmentsQuery = useQuery({
    queryKey: createQueryKey("attendance-policy-assignments", page, limit, options.q, serverTargetType),
    queryFn: () => fetchPolicyAssignments({ page, limit, q: options.q, target_type: serverTargetType }),
  });
  const assignments = useMemo(() => assignmentsQuery.data?.data ?? [], [assignmentsQuery.data?.data]);
  const { data: departments = [] } = useOrganizationQuery({ queryKey: departmentKeys.all(), queryFn: fetchDepartments });
  const { data: employeesResponse } = useOrganizationQuery({ queryKey: employeeKeys.lookup(), queryFn: () => fetchEmployees(1, 300) });
  const employees = useMemo(() => employeesResponse?.data ?? [], [employeesResponse]);

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deletePolicyAssignment(id),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-policy-assignments") });
    },
  });
  const upsertMutation = useMutation({
    mutationFn: (data: UpsertPolicyAssignmentRequest) => upsertPolicyAssignment(data),
    onSuccess: () => {
      toast.success(t("attendance_policy_overrides.messages.save_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-policy-assignments") });
    },
  });

  const departmentMap = useMemo(() => new Map(departments.map((department) => [department.id, department.name])), [departments]);
  const employeeMap = useMemo(() => new Map(employees.map((employee: Employee) => [employee.id, employee.full_name])), [employees]);
  const resolveTargetName = useCallback(
    (type: string, id: string) => getTargetName({ assignments, departments: departmentMap, employees: employeeMap, type, id }),
    [assignments, departmentMap, employeeMap],
  );
  const workUnitLabel = useCallback((value: number) => t(`common.organization_settings.work_unit_options.${value}min`, { defaultValue: `${value} min` }), [t]);
  const roundingLabel = useCallback((value: string) => t(`common.organization_settings.rounding_options.${value}`, { defaultValue: value }), [t]);

  const handleSave = () => {
    if (!targetId) {
      toast.error(t("attendance_policy_overrides.messages.validation_select_target"));
      return;
    }
    upsertMutation.mutate(buildAssignmentRequest({ targetType, targetId, workUnitMinutes, roundingPolicy, earlyArrivalStrategy, validFrom, gracePeriodMinutes }), {
      onSuccess: () => {
        setIsDrawerOpen(false);
        setTargetId("");
        setGracePeriodMinutes(undefined);
      },
    });
  };

  return {
    assignments,
    departments,
    employees,
    isLoadingAssignments: assignmentsQuery.isLoading,
    isAssignmentsError: assignmentsQuery.isError,
    refetchAssignments: assignmentsQuery.refetch,
    totalAssignments: assignmentsQuery.data?.total ?? 0,
    page,
    limit,
    deleteMutation,
    upsertMutation,
    activeFilter,
    setActiveFilter: (filter: FilterType) => {
      setLocalFilter(filter);
      options.onTargetTypeChange?.(filter === "ALL" ? undefined : filter);
    },
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
    filteredAssignments: assignments,
    getTargetName: resolveTargetName,
    workUnitLabel,
    roundingLabel,
    handleSave,
  };
}

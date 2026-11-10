import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useDepartments } from "@/hooks/use-departments";

import { useToast } from "@/components/ui/toast";
import {
  fetchPayrollRuleAssignments,
  createPayrollRuleAssignment,
  deletePayrollRuleAssignment,
} from "@/lib/api/payroll";
import { PayrollRuleAssignment, Department } from "@/lib/api/types";

type AssigneeType = "organization" | "department" | "employee";

export function useRuleAssignments(ruleId: string) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const assignmentsQuery = useQuery({
    queryKey: ["payroll-rule-assignments", ruleId],
    queryFn: async () => {
      const res = await fetchPayrollRuleAssignments(ruleId);
      const maybeRes = ((): typeof res | undefined => res)();
      return maybeRes ?? [];
    },
    enabled: !!ruleId,
  });

  const departmentsQuery = useDepartments();

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createPayrollRuleAssignment>[1]) =>
      createPayrollRuleAssignment(ruleId, data),
    onSuccess: () => {
      toast.success(t("common.success", "Success"));
      void queryClient.invalidateQueries({
        queryKey: ["payroll-rule-assignments", ruleId],
      });
    },
    onError: (error: Error) => { toast.error(error.message || t("common.error", "Error")); },
  });

  const deleteMutation = useMutation({
    mutationFn: (assignmentId: string) =>
      deletePayrollRuleAssignment(ruleId, assignmentId),
    onSuccess: () => {
      toast.success(t("common.success", "Success"));
      void queryClient.invalidateQueries({
        queryKey: ["payroll-rule-assignments", ruleId],
      });
    },
    onError: (error: Error) => { toast.error(error.message || t("common.error", "Error")); },
  });

  const deptMap = useMemo(() => {
    const m: Record<string, Department> = {};
    departmentsQuery.data?.forEach((d) => {
      m[d.id] = d;
    });
    return m;
  }, [departmentsQuery.data]);

  const grouped = useMemo(() => {
    const g: Record<AssigneeType, PayrollRuleAssignment[]> = {
      organization: [],
      department: [],
      employee: [],
    };
    assignmentsQuery.data?.forEach((assignment) => {
      const assigneeType = assignment.assignee_type as AssigneeType;
      if (assigneeType in g) g[assigneeType].push(assignment);
    });
    return g;
  }, [assignmentsQuery.data]);

  return {
    assignments: assignmentsQuery.data,
    isLoading: assignmentsQuery.isLoading,
    departments: departmentsQuery.data,
    deptMap,
    grouped,
    totalCount: assignmentsQuery.data?.length ?? 0,
    createMutation,
    deleteMutation,
  };
}

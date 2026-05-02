import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { fetchDepartments } from "@/lib/api/departments";
import {
  fetchPayrollRuleAssignments,
  createPayrollRuleAssignment,
  deletePayrollRuleAssignment,
} from "@/lib/api/payroll";
import { PayrollRuleAssignment, Department } from "@/lib/api/types";

export function useRuleAssignments(wsId: string, ruleId: string) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const assignmentsQuery = useQuery({
    queryKey: ["payroll-rule-assignments", ruleId],
    queryFn: () => fetchPayrollRuleAssignments(wsId, ruleId),
    enabled: !!ruleId,
  });

  const departmentsQuery = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<PayrollRuleAssignment>) =>
      createPayrollRuleAssignment(wsId, ruleId, data),
    onSuccess: () => {
      toast.success(t("common.success", "Success"));
      queryClient.invalidateQueries({
        queryKey: ["payroll-rule-assignments", ruleId],
      });
    },
    onError: (error: Error) => { toast.error(error.message || t("common.error", "Error")); },
  });

  const deleteMutation = useMutation({
    mutationFn: (assignmentId: string) =>
      deletePayrollRuleAssignment(wsId, ruleId, assignmentId),
    onSuccess: () => {
      toast.success(t("common.success", "Success"));
      queryClient.invalidateQueries({
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
    const g = {
      workspace: [] as PayrollRuleAssignment[],
      department: [] as PayrollRuleAssignment[],
      employee: [] as PayrollRuleAssignment[],
    };
    assignmentsQuery.data?.forEach((a) => g[a.assignee_type]?.push(a));
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

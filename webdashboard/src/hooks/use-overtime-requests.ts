import { useQuery, useQueryClient } from '@tanstack/react-query';

import { useOrganizationQuery, useOrganizationMutation } from '@/hooks/common/use-workspace-query';
import { useWorkspace } from '@/hooks/use-workspace';
import { overtimeRequestsApi, CreateOvertimeRequestPayload, ConsentStatus } from '@/lib/api/overtime-requests';

export const overtimeRequestKeys = {
  all: ['overtime-requests'] as const,
  lists: (workspaceId: string) => [...overtimeRequestKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, params?: Record<string, unknown>) => [...overtimeRequestKeys.lists(workspaceId), { params }] as const,
  details: () => [...overtimeRequestKeys.all, 'detail'] as const,
  detail: (workspaceId: string, id: string) => [...overtimeRequestKeys.details(), workspaceId, id] as const,
  assignments: (employmentId: string) => [...overtimeRequestKeys.all, 'assignments', employmentId] as const,
  quotas: () => ['overtime-quotas'] as const,
  quotaSummaries: (workspaceId: string, month: string) => [...overtimeRequestKeys.quotas(), workspaceId, 'summary', month] as const,
  quotaDetail: (workspaceId: string, employeeId: string, date: string) => [...overtimeRequestKeys.quotas(), workspaceId, employeeId, date] as const,
};

export const useOvertimeRequests = (params?: Record<string, unknown>) => {
  const { workspace } = useWorkspace();
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.list(workspace?.id ?? '', params),
    queryFn: (wsId) => overtimeRequestsApi.listRequests(wsId, params),
  });
};

export const useOvertimeRequest = (id: string) => {
  const { workspace } = useWorkspace();
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.detail(workspace?.id ?? '', id),
    queryFn: (wsId) => overtimeRequestsApi.getRequest(wsId, id),
    enabled: !!id,
  });
};

export const useCreateOvertimeRequest = () =>
  useOrganizationMutation<CreateOvertimeRequestPayload>({
    mutationFn: (wsId, payload) => overtimeRequestsApi.createRequest(wsId, payload),
    keys: [overtimeRequestKeys.all],
  });

export const useSubmitOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();
  return useOrganizationMutation<string>({
    mutationFn: (wsId, id) => overtimeRequestsApi.submitRequest(wsId, id),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, id) => {
      if (workspace?.id) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace.id, id) });
      }
    },
  });
};

export const useCancelOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();
  return useOrganizationMutation<string>({
    mutationFn: (wsId, id) => overtimeRequestsApi.cancelRequest(wsId, id),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, id) => {
      if (workspace?.id) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace.id, id) });
      }
    },
  });
};

export const useEmployeeConsent = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();
  return useOrganizationMutation<{ id: string; employment_id: string; status: ConsentStatus; decline_reason?: string }>({
    mutationFn: (wsId, vars) =>
      overtimeRequestsApi.employeeConsent(wsId, vars.id, {
        employment_id: vars.employment_id,
        status: vars.status,
        decline_reason: vars.decline_reason,
      }),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, variables) => {
      if (workspace?.id) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace.id, variables.id) });
      }
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.assignments(variables.employment_id) });
    },
  });
};

export const useMyOtAssignments = (employmentId?: string) => {
  return useQuery({
    queryKey: overtimeRequestKeys.assignments(employmentId ?? ''),
    queryFn: () => {
      if (!employmentId) throw new Error('No employment ID');
      return overtimeRequestsApi.getMyAssignments(employmentId);
    },
    enabled: !!employmentId,
  });
};

export const useWorkspaceQuotaSummary = (month: string) => {
  const { workspace } = useWorkspace();
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.quotaSummaries(workspace?.id ?? '', month),
    queryFn: (wsId) => overtimeRequestsApi.getWorkspaceQuotaSummary(wsId, month),
    enabled: !!month,
  });
};

export const useEmployeeQuota = (employeeId: string, date: string) => {
  const { workspace } = useWorkspace();
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.quotaDetail(workspace?.id ?? '', employeeId, date),
    queryFn: (wsId) => overtimeRequestsApi.getEmployeeQuota(wsId, employeeId, date),
    enabled: !!employeeId && !!date,
  });
};

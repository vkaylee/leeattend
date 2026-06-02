import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

import { useWorkspace } from '@/hooks/use-workspace';
import { overtimeRequestsApi, CreateOvertimeRequestPayload, ConsentStatus } from '@/lib/api/overtime-requests';

export const overtimeRequestKeys = {
  all: ['overtime-requests'] as const,
  lists: (workspaceId: string) => [...overtimeRequestKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, params?: Record<string, unknown>) => [...overtimeRequestKeys.lists(workspaceId), { params }] as const,
  details: () => [...overtimeRequestKeys.all, 'detail'] as const,
  detail: (workspaceId: string, id: string) => [...overtimeRequestKeys.details(), workspaceId, id] as const,
  assignments: (employeeId: string) => [...overtimeRequestKeys.all, 'assignments', employeeId] as const,
  quotas: () => ['overtime-quotas'] as const,
  quotaSummaries: (workspaceId: string, month: string) => [...overtimeRequestKeys.quotas(), workspaceId, 'summary', month] as const,
  quotaDetail: (workspaceId: string, employeeId: string, date: string) => [...overtimeRequestKeys.quotas(), workspaceId, employeeId, date] as const,
};

export const useOvertimeRequests = (params?: Record<string, unknown>) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: overtimeRequestKeys.list(workspaceId ?? '', params),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.listRequests(workspaceId, params);
    },
    enabled: !!workspaceId,
  });
};

export const useOvertimeRequest = (id: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: overtimeRequestKeys.detail(workspaceId ?? '', id),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.getRequest(workspaceId, id);
    },
    enabled: !!workspaceId && !!id,
  });
};

export const useCreateOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateOvertimeRequestPayload) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.createRequest(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
    },
  });
};

export const useSubmitOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.submitRequest(workspaceId, id);
    },
    onSuccess: (_, id) => {
      const workspaceId = workspace?.id;
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      if (workspaceId) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspaceId, id) });
      }
    },
  });
};

export const useCancelOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.cancelRequest(workspaceId, id);
    },
    onSuccess: (_, id) => {
      const workspaceId = workspace?.id;
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      if (workspaceId) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspaceId, id) });
      }
    },
  });
};

export const useEmployeeConsent = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ id, employee_id, status, decline_reason }: { id: string; employee_id: string; status: ConsentStatus; decline_reason?: string }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.employeeConsent(workspaceId, id, { employee_id, status, decline_reason });
    },
    onSuccess: (_, variables) => {
      const workspaceId = workspace?.id;
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      if (workspaceId) {
        void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspaceId, variables.id) });
      }
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.assignments(variables.employee_id) });
    },
  });
};

export const useMyOtAssignments = (employeeId?: string) => {
  return useQuery({
    queryKey: overtimeRequestKeys.assignments(employeeId ?? ''),
    queryFn: () => {
      if (!employeeId) throw new Error('No employee ID');
      return overtimeRequestsApi.getMyAssignments(employeeId);
    },
    enabled: !!employeeId,
  });
};

export const useWorkspaceQuotaSummary = (month: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: overtimeRequestKeys.quotaSummaries(workspaceId ?? '', month),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.getWorkspaceQuotaSummary(workspaceId, month);
    },
    enabled: !!workspaceId && !!month,
  });
};

export const useEmployeeQuota = (employeeId: string, date: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: overtimeRequestKeys.quotaDetail(workspaceId ?? '', employeeId, date),
    queryFn: () => {
      if (!workspaceId) throw new Error('No workspace ID');
      return overtimeRequestsApi.getEmployeeQuota(workspaceId, employeeId, date);
    },
    enabled: !!workspaceId && !!employeeId && !!date,
  });
};

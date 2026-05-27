import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { overtimeRequestsApi, CreateOvertimeRequestPayload, ConsentStatus } from '@/lib/api/overtime-requests';
import { useWorkspace } from '@/hooks/use-workspace';

export const overtimeRequestKeys = {
  all: ['overtime-requests'] as const,
  lists: (workspaceId: string) => [...overtimeRequestKeys.all, workspaceId, 'list'] as const,
  list: (workspaceId: string, params?: any) => [...overtimeRequestKeys.lists(workspaceId), { params }] as const,
  details: () => [...overtimeRequestKeys.all, 'detail'] as const,
  detail: (workspaceId: string, id: string) => [...overtimeRequestKeys.details(), workspaceId, id] as const,
  assignments: (employeeId: string) => [...overtimeRequestKeys.all, 'assignments', employeeId] as const,
  quotas: () => ['overtime-quotas'] as const,
  quotaSummaries: (workspaceId: string, month: string) => [...overtimeRequestKeys.quotas(), workspaceId, 'summary', month] as const,
  quotaDetail: (workspaceId: string, employeeId: string, date: string) => [...overtimeRequestKeys.quotas(), workspaceId, employeeId, date] as const,
};

export const useOvertimeRequests = (params?: any) => {
  const { workspace } = useWorkspace();

  return useQuery({
    queryKey: overtimeRequestKeys.list(workspace?.id || '', params),
    queryFn: () => overtimeRequestsApi.listRequests(workspace!.id, params),
    enabled: !!workspace?.id,
  });
};

export const useOvertimeRequest = (id: string) => {
  const { workspace } = useWorkspace();

  return useQuery({
    queryKey: overtimeRequestKeys.detail(workspace?.id || '', id),
    queryFn: () => overtimeRequestsApi.getRequest(workspace!.id, id),
    enabled: !!workspace?.id && !!id,
  });
};

export const useCreateOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateOvertimeRequestPayload) => 
      overtimeRequestsApi.createRequest(workspace!.id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
    },
  });
};

export const useSubmitOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => 
      overtimeRequestsApi.submitRequest(workspace!.id, id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace!.id, id) });
    },
  });
};

export const useCancelOvertimeRequest = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => 
      overtimeRequestsApi.cancelRequest(workspace!.id, id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace!.id, id) });
    },
  });
};

export const useEmployeeConsent = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ id, employee_id, status, decline_reason }: { id: string; employee_id: string; status: ConsentStatus; decline_reason?: string }) => 
      overtimeRequestsApi.employeeConsent(workspace!.id, id, { employee_id, status, decline_reason }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.all });
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(workspace!.id, variables.id) });
      queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.assignments(variables.employee_id) });
    },
  });
};

export const useMyOtAssignments = (employeeId?: string) => {
  return useQuery({
    queryKey: overtimeRequestKeys.assignments(employeeId || ''),
    queryFn: () => overtimeRequestsApi.getMyAssignments(employeeId!),
    enabled: !!employeeId,
  });
};

export const useWorkspaceQuotaSummary = (month: string) => {
  const { workspace } = useWorkspace();

  return useQuery({
    queryKey: overtimeRequestKeys.quotaSummaries(workspace?.id || '', month),
    queryFn: () => overtimeRequestsApi.getWorkspaceQuotaSummary(workspace!.id, month),
    enabled: !!workspace?.id && !!month,
  });
};

export const useEmployeeQuota = (employeeId: string, date: string) => {
  const { workspace } = useWorkspace();

  return useQuery({
    queryKey: overtimeRequestKeys.quotaDetail(workspace?.id || '', employeeId, date),
    queryFn: () => overtimeRequestsApi.getEmployeeQuota(workspace!.id, employeeId, date),
    enabled: !!workspace?.id && !!employeeId && !!date,
  });
};

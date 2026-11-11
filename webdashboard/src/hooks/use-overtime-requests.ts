import { useQuery, useQueryClient } from '@tanstack/react-query';

import { useOrganizationQuery, useOrganizationMutation } from '@/hooks/common/use-organization-query';
import { overtimeRequestsApi, CreateOvertimeRequestPayload, ConsentStatus } from '@/lib/api/overtime-requests';

export const overtimeRequestKeys = {
  all: ['overtime-requests'] as const,
  lists: () => [...overtimeRequestKeys.all, 'list'] as const,
  list: (params?: Record<string, unknown>) => [...overtimeRequestKeys.lists(), { params }] as const,
  details: () => [...overtimeRequestKeys.all, 'detail'] as const,
  detail: (id: string) => [...overtimeRequestKeys.details(), id] as const,
  assignments: (employmentId: string) => [...overtimeRequestKeys.all, 'assignments', employmentId] as const,
};

export const useOvertimeRequests = (params?: Record<string, unknown>) => {
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.list(params),
    queryFn: () => overtimeRequestsApi.listRequests(params),
  });
};

export const useOvertimeRequest = (id: string) => {
  return useOrganizationQuery({
    queryKey: overtimeRequestKeys.detail(id),
    queryFn: () => overtimeRequestsApi.getRequest(id),
    enabled: !!id,
  });
};

export const useCreateOvertimeRequest = () =>
  useOrganizationMutation<CreateOvertimeRequestPayload>({
    mutationFn: (payload) => overtimeRequestsApi.createRequest(payload),
    keys: [overtimeRequestKeys.all],
  });

export const useSubmitOvertimeRequest = () => {
  const queryClient = useQueryClient();
  return useOrganizationMutation<string>({
    mutationFn: (id) => overtimeRequestsApi.submitRequest(id),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, id) => {
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(id) });
    },
  });
};

export const useCancelOvertimeRequest = () => {
  const queryClient = useQueryClient();
  return useOrganizationMutation<string>({
    mutationFn: (id) => overtimeRequestsApi.cancelRequest(id),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, id) => {
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(id) });
    },
  });
};

export const useEmployeeConsent = () => {
  const queryClient = useQueryClient();
  return useOrganizationMutation<{ id: string; employment_id: string; status: ConsentStatus; decline_reason?: string }>({
    mutationFn: (vars) =>
      overtimeRequestsApi.employeeConsent(vars.id, {
        employment_id: vars.employment_id,
        status: vars.status,
        decline_reason: vars.decline_reason,
      }),
    keys: [overtimeRequestKeys.all],
    onSuccess: (_, variables) => {
      void queryClient.invalidateQueries({ queryKey: overtimeRequestKeys.detail(variables.id) });
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

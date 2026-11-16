import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import {
  CreateEmergencyContactRequest,
  EmployeeEmergencyContact,
  Employee,
  MyProfileResponse,
  EmployeeProfile,
  UpdateProfileRequest,
  addEmergencyContact,
  deleteEmergencyContact,
  fetchMyEmergencyContacts,
  fetchMyProfile,
  updateMyProfile,
} from "@/lib/api";

import { essKeys } from "./use-ess-keys";

export interface UseMyProfileReturn {
  data: MyProfileResponse | undefined;
  emergencyContacts: EmployeeEmergencyContact[];
  employee: Employee | undefined;
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  isUpdating: boolean;
  profile: EmployeeProfile | undefined;
  refetch: () => Promise<unknown>;
  updateProfile: (data: UpdateProfileRequest) => Promise<EmployeeProfile>;
}

export function useMyProfile(options: { enabled?: boolean } = {}): UseMyProfileReturn {
  const queryClient = useQueryClient();

  const profileQuery = useQuery<MyProfileResponse>({
    queryKey: essKeys.profile(),
    queryFn: fetchMyProfile,
    enabled: options.enabled ?? true,
    meta: { suppressErrorToast: [403] },
  });

  const updateMutation = useMutation<EmployeeProfile, Error, UpdateProfileRequest>({
    mutationFn: (data: UpdateProfileRequest) => updateMyProfile(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    data: profileQuery.data,
    employee: profileQuery.data?.employee,
    profile: profileQuery.data?.profile ?? undefined,
    emergencyContacts: profileQuery.data?.emergency_contacts ?? [],
    isLoading: profileQuery.isLoading,
    isError: profileQuery.isError,
    error: profileQuery.error,
    refetch: profileQuery.refetch,

    updateProfile: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
  };
}

export interface UseEmergencyContactsReturn {
  addContact: (data: CreateEmergencyContactRequest) => Promise<EmployeeEmergencyContact>;
  contacts: EmployeeEmergencyContact[];
  deleteContact: (contactId: string) => Promise<unknown>;
  error: Error | null;
  isAdding: boolean;
  isDeleting: boolean;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useEmergencyContacts(): UseEmergencyContactsReturn {
  const queryClient = useQueryClient();

  const contactsQuery = useQuery<EmployeeEmergencyContact[]>({
    queryKey: essKeys.emergencyContacts(),
    queryFn: fetchMyEmergencyContacts,
  });

  const addMutation = useMutation<EmployeeEmergencyContact, Error, CreateEmergencyContactRequest>({
    mutationFn: (data: CreateEmergencyContactRequest) =>
      addEmergencyContact(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  const deleteMutation = useMutation<unknown, Error, string>({
    mutationFn: (contactId: string) => deleteEmergencyContact(contactId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    contacts: contactsQuery.data ?? [],
    isLoading: contactsQuery.isLoading,
    isError: contactsQuery.isError,
    error: contactsQuery.error,
    refetch: contactsQuery.refetch,

    addContact: addMutation.mutateAsync,
    isAdding: addMutation.isPending,

    deleteContact: deleteMutation.mutateAsync,
    isDeleting: deleteMutation.isPending,
  };
}

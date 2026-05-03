import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import { fetchMe, updateMe, setCurrentUserId } from "@/lib/api";

export const profileKeys = {
  me: ["profile-me"] as const,
};

export function useProfile() {
  const queryClient = useQueryClient();

  const profileQuery = useQuery({
    queryKey: profileKeys.me,
    queryFn: async () => {
      const data = await fetchMe();
      // Sync local storage user ID for dev mode
      setCurrentUserId(data.id);
      return data;
    },
  });

  const updateMutation = useMutation({
    mutationFn: updateMe,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: profileKeys.me });
    },
  });

  return {
    user: profileQuery.data,
    isLoading: profileQuery.isLoading,
    isError: profileQuery.isError,
    error: profileQuery.error,
    refetch: profileQuery.refetch,

    updateProfile: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
  };
}

import { useQuery } from "@tanstack/react-query";

import { fetchSystemTenants } from "@/lib/api";

export function useSystemTenants() {
  const query = useQuery({
    queryKey: ["system-tenants"],
    queryFn: fetchSystemTenants,
  });

  return {
    tenants: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
  };
}

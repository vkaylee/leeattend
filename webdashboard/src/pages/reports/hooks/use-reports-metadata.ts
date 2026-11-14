import { useQuery } from "@tanstack/react-query";

import { useDepartments } from "@/hooks/use-departments";
import { fetchOrganization, fetchEmployers } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


export function useReportsMetadata() {
  const { data: organization } = useQuery({
    queryKey: createQueryKey("organization"),
    queryFn: () => fetchOrganization(),
  });

  const { data: departments } = useDepartments();

  const { data: employers } = useQuery({
    queryKey: createQueryKey("employers"),
    queryFn: () => fetchEmployers(),
  });

  return { organization, departments, employers };
}

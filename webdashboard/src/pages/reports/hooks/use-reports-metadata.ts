import { useQuery } from "@tanstack/react-query";

import { useDepartments } from "@/hooks/use-departments";
import { fetchOrganization, fetchEmployers } from "@/lib/api";

export function useReportsMetadata() {
  const { data: organization } = useQuery({
    queryKey: ["organization"],
    queryFn: () => fetchOrganization(),
  });

  const { data: departments } = useDepartments();

  const { data: employers } = useQuery({
    queryKey: ["employers"],
    queryFn: () => fetchEmployers(),
  });

  return { organization, departments, employers };
}

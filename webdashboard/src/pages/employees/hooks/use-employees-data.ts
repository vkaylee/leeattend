import { useMemo } from "react";

import { useOrganizationInfiniteQuery } from "@/hooks/common/use-organization-query";
import { useDepartments } from "@/hooks/use-departments";
import { useMyPermissions } from "@/hooks/use-permissions";
import { searchEmployees } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";

import type { SortKey, SortDir } from "./use-employees-state";


const PAGE_SIZE = 20;

interface UseEmployeesDataProps { customFieldFilters: Record<string, string>;
debouncedSearch: string;
filterDeptIds: string[];
filterStatus: string;
sortDir: SortDir;
sortKey: SortKey | null;  }

export const useEmployeesData = ({
  debouncedSearch,
  sortKey,
  sortDir,
  filterDeptIds,
  filterStatus,
  customFieldFilters,
}: UseEmployeesDataProps) => {
  const { hasPermission } = useMyPermissions();
  const canReadEmployees = hasPermission("employees:read");

  const {
    data: employeesPages,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useOrganizationInfiniteQuery({ queryKey: employeeKeys.search({
    customFieldFilters,
    departmentIds: filterDeptIds,
    search: debouncedSearch,
    sortDir,
    sortKey,
    status: filterStatus,
  }),
  queryFn: (pageParam) => {
    const apiSortKeyMapping = { name: "name", id: "employee_id_number", department: "department" } as const;
    const apiSortKey = sortKey ? apiSortKeyMapping[sortKey] : undefined;
    const cfString = Object.entries(customFieldFilters)
      .filter(([_, val]) => !!val)
      .map(([key, val]) => `${key}:${val}`)
      .join(",");
    return searchEmployees({
      query: debouncedSearch,
      page: pageParam,
      limit: PAGE_SIZE,
      sortBy: apiSortKey,
      sortDir,
      departmentIds: filterDeptIds.length > 0 ? filterDeptIds : undefined,
      customFieldFilters: cfString || undefined,
      status: filterStatus,
    });
  },
  getNextPageParam: (lastPage, allPages) => {
    const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
    if (totalLoaded >= (lastPage.total || 0)) return undefined;
    return allPages.length + 1;
  },
  initialPageParam: 1,
  enabled: canReadEmployees,
  placeholderData: "keepPreviousData", });

  const employees = useMemo(() => {
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => p.data);
  }, [employeesPages]);

  const totalItems = employeesPages?.pages[0]?.total ?? 0;

  const { data: departments } = useDepartments();

  return {
    employees,
    totalItems,
    departments,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  };
};

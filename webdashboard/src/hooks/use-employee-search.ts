import { useOrganizationInfiniteQuery, useOrganizationQuery } from '@/hooks/common/use-organization-query';
import { employeeKeys } from '@/lib/query-keys';
import { useState, useMemo, useEffect } from 'react';

import { searchEmployees, fetchEmployee } from '../lib/api';
import { Employee, PaginatedResponse } from '../lib/api/types';

export interface UseEmployeeSearchOptions {
  debouncedSearchTerm: string;
  departmentIds?: string[];
  emptyQueryBehavior?: 'prompt-to-type' | 'all';
  permission?: string;
  selectedIds: string[];
}

export function useEmployeeSearch({
  debouncedSearchTerm,
  selectedIds,
  emptyQueryBehavior = 'all',
  permission,
  departmentIds,
}: UseEmployeeSearchOptions) {
  const isQueryEmpty = debouncedSearchTerm.trim() === '';
  const isEnabled = emptyQueryBehavior === 'all' || !isQueryEmpty;

  const {
    data: employeesPages,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
  } = useOrganizationInfiniteQuery({
    queryKey: employeeKeys.search({
      departmentIds,
      emptyQueryBehavior,
      permission,
      search: debouncedSearchTerm,
    }),
    queryFn: async (pageParam) => {
      const res = (await searchEmployees({
        query: debouncedSearchTerm,
        page: pageParam,
        limit: 50,
        permission,
        departmentIds,
      })) as PaginatedResponse<Employee> | undefined;
      return res ?? { data: [], limit: 50, page: pageParam, total: 0 };
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p?.data?.length ?? 0), 0);
      if (totalLoaded >= (lastPage?.total ?? 0)) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: isEnabled,
  });

  const employees = useMemo(() => {
    if (emptyQueryBehavior === 'prompt-to-type' && isQueryEmpty) {
      return [];
    }
    if (!employeesPages) return [];
    return employeesPages.pages.flatMap((p) => Array.isArray(p.data) ? p.data : []);
  }, [employeesPages, emptyQueryBehavior, isQueryEmpty]);

  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

  const firstSelectedId = selectedIds[0];
  const { data: selectedEmpData } = useOrganizationQuery({
    queryKey: employeeKeys.detail(firstSelectedId ?? ""),
    queryFn: () => {
      if (!firstSelectedId) throw new Error("No selected ID");
      return fetchEmployee(firstSelectedId);
    },
    enabled: !!firstSelectedId && !selectedMeta.some(m => m.id === firstSelectedId),
  });

  useEffect(() => {
    if (selectedEmpData) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSelectedMeta(prev => {
        if (prev.some(m => m.id === selectedEmpData.id)) return prev;
        return [...prev, selectedEmpData];
      });
    }
  }, [selectedEmpData]);

  useEffect(() => {
    if (employees.length === 0 || selectedIds.length === 0) return;
    const found = employees.filter(emp => selectedIds.includes(emp.id));
    if (found.length === 0) return;

    // eslint-disable-next-line react-hooks/set-state-in-effect
    setSelectedMeta(previous => {
      const missing = found.filter(f => !previous.some(m => m.id === f.id));
      if (missing.length === 0) return previous;
      return [...previous, ...missing];
    });
  }, [employees, selectedIds]);

  return {
    employees,
    selectedMeta,
    setSelectedMeta,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
  };
}

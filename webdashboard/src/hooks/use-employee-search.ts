import { useQuery, useInfiniteQuery } from '@tanstack/react-query';
import { useState, useMemo, useEffect } from 'react';

import { searchEmployees, fetchEmployee } from '../lib/api';
import { Employee, PaginatedResponse } from '../lib/api/types';

export interface UseEmployeeSearchOptions {
  debouncedSearchTerm: string;
  departmentIds?: string[];
  emptyQueryBehavior?: 'prompt-to-type' | 'all';
  permission?: string;
  selectedIds: string[];
  workspaceId: string;
}

export function useEmployeeSearch({
  workspaceId,
  debouncedSearchTerm,
  selectedIds,
  emptyQueryBehavior = 'all',
  permission,
  departmentIds,
}: UseEmployeeSearchOptions) {
  const isQueryEmpty = debouncedSearchTerm.trim() === '';
  const isEnabled = workspaceId !== '' && (emptyQueryBehavior === 'all' || !isQueryEmpty);

  const {
    data: employeesPages,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
  } = useInfiniteQuery({
    queryKey: ['employees', workspaceId, 'search', debouncedSearchTerm, emptyQueryBehavior, permission, departmentIds?.join(',') ?? ''],
    queryFn: async ({ pageParam }) => {
      const res = (await searchEmployees({
        wsId: workspaceId,
        query: debouncedSearchTerm,
        page: pageParam,
        limit: 50,
        permission,
        departmentIds,
      })) as PaginatedResponse<Employee> | undefined;
      return res ?? { data: [], limit: 50, page: pageParam, total: 0 };
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + p.data.length, 0);
      if (totalLoaded >= lastPage.total) return undefined;
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
    return employeesPages.pages.flatMap((p) => p.data);
  }, [employeesPages, emptyQueryBehavior, isQueryEmpty]);

  const [selectedMeta, setSelectedMeta] = useState<Employee[]>([]);

  const firstSelectedId = selectedIds[0];
  const { data: selectedEmpData } = useQuery({
    queryKey: ['employee', workspaceId, firstSelectedId],
    queryFn: () => {
      if (!firstSelectedId) throw new Error("No selected ID");
      return fetchEmployee(workspaceId, firstSelectedId);
    },
    enabled: workspaceId !== '' && !!firstSelectedId && !selectedMeta.some(m => m.id === firstSelectedId),
  });

  useEffect(() => {
    if (selectedEmpData) {
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

import { useQuery } from "@tanstack/react-query";

import {
  EmployeeDocument,
  Payslip,
  fetchMyDocuments,
  fetchMyPayslips,
} from "@/lib/api";

import { essKeys } from "./use-ess-keys";

export interface UseMyDocumentsReturn {
  documents: EmployeeDocument[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useMyDocuments(): UseMyDocumentsReturn {
  const documentsQuery = useQuery<EmployeeDocument[]>({
    queryKey: essKeys.documents(),
    queryFn: fetchMyDocuments,
  });

  return {
    documents: documentsQuery.data ?? [],
    isLoading: documentsQuery.isLoading,
    isError: documentsQuery.isError,
    error: documentsQuery.error,
    refetch: documentsQuery.refetch,
  };
}

export interface UseMyPayslipsReturn {
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  payslips: Payslip[];
  refetch: () => Promise<unknown>;
}

export function useMyPayslips(): UseMyPayslipsReturn {
  const payslipsQuery = useQuery<Payslip[]>({
    queryKey: essKeys.payslips(),
    queryFn: fetchMyPayslips,
  });

  return {
    payslips: payslipsQuery.data ?? [],
    isLoading: payslipsQuery.isLoading,
    isError: payslipsQuery.isError,
    error: payslipsQuery.error,
    refetch: payslipsQuery.refetch,
  };
}

export type ContractType = "probation" | "training" | "fixed_term" | "indefinite";
export type ContractStatus = "draft" | "active" | "expired" | "terminated";

export interface EmployeeContract {
  contract_number?: string | null;
  contract_type: ContractType;
  created_at: string;
  document_id?: string | null;
  employee_id: string;
  end_date?: string | null;
  id: string;
  notes?: string | null;
  start_date: string;
  status: ContractStatus;
  updated_at: string;
  workspace_id: string;
}

export interface CreateContractRequest {
  contract_number?: string | null;
  contract_type: ContractType;
  document_id?: string | null;
  employee_id: string;
  end_date?: string | null;
  notes?: string | null;
  start_date: string;
  status?: ContractStatus | null;
}

export interface UpdateContractRequest {
  contract_number?: string | null;
  contract_type?: ContractType | null;
  document_id?: string | null;
  end_date?: string | null;
  notes?: string | null;
  start_date?: string | null;
  status?: ContractStatus | null;
}

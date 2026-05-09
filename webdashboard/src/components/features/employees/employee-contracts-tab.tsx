import { Plus, FileText, Pencil, Trash2 } from "lucide-react";
import React, { useState } from "react";

import type { EmployeeContract, ContractType, ContractStatus } from "@/lib/api";

import PermissionGate from "@/components/common/permission-gate";
import { cn } from "@/lib/utils";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

import ContractFormModal from "./contract-form-modal";

interface EmployeeContractsTabProps {
  employeeId: string;
  formatDate: (date: string) => string;
  wsId: string;
}

// eslint-disable-next-line max-lines-per-function
export default function EmployeeContractsTab({ wsId, employeeId, formatDate }: EmployeeContractsTabProps) {
  const { contracts, isLoading, deleteContract } = useEmployeeContracts(wsId, employeeId);
  const [editingContract, setEditingContract] = useState<EmployeeContract | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  if (isLoading) {
    return (
      <div className="space-y-4 animate-pulse pt-4">
        <div className="h-20 bg-slate-100 dark:bg-slate-800 rounded-2xl w-full"></div>
        <div className="h-20 bg-slate-100 dark:bg-slate-800 rounded-2xl w-full"></div>
      </div>
    );
  }

  const getContractTypeLabel = (type: ContractType) => {
    switch (type) {
      case "probation": return "Thử việc";
      case "training": return "Đào tạo";
      case "fixed_term": return "Xác định thời hạn";
      case "indefinite": return "Không xác định thời hạn";
      default: return type;
    }
  };

  const getStatusColor = (status: ContractStatus) => {
    switch (status) {
      case "active": return "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950 dark:text-emerald-300 dark:border-emerald-800";
      case "expired": return "bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950 dark:text-rose-300 dark:border-rose-800";
      case "terminated": return "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700";
      case "draft": return "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950 dark:text-amber-300 dark:border-amber-800";
      default: return "bg-slate-50 text-slate-500";
    }
  };

  return (
    <div className="space-y-6 pt-4">
      <div className="flex justify-between items-center px-2">
        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-2 opacity-70">
          <FileText className="text-primary-500" size={14} /> Hợp đồng lao động
        </span>
        <PermissionGate permission="employees:update">
          <button 
            className="flex items-center gap-1.5 text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:bg-primary-50 dark:hover:bg-primary-900/50 py-1.5 px-3 rounded-full transition-colors"
            onClick={() => {
              setEditingContract(null);
              setIsModalOpen(true);
            }}
          >
            <Plus size={14} /> Ký hợp đồng
          </button>
        </PermissionGate>
      </div>

      <div className="space-y-4">
        {contracts.length === 0 ? (
          <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] flex flex-col items-center justify-center text-center shadow-inner group hover:bg-white dark:hover:bg-slate-900 transition-all">
             <FileText className="w-12 h-12 text-slate-300 dark:text-slate-600 mb-3 group-hover:text-primary-400 transition-colors" />
             <div className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">Chưa có hợp đồng nào</div>
          </div>
        ) : (
          contracts.map((contract: EmployeeContract) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm hover:shadow-xl hover:shadow-primary-100/10 transition-all group overflow-hidden relative" key={contract.id}>
              <div className="flex justify-between items-start mb-4">
                <div>
                  <div className="flex items-center gap-3 mb-1.5">
                    <div className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">{getContractTypeLabel(contract.contract_type)}</div>
                    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 rounded-full border shadow-sm", getStatusColor(contract.status))}>
                      {contract.status}
                    </span>
                  </div>
                  {!!contract.contract_number && <div className="text-[10px] font-mono text-slate-500 dark:text-slate-400">Số HĐ: {contract.contract_number}</div>}
                </div>
                <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                   <button 
                      className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-colors"
                      onClick={() => {
                        setEditingContract(contract);
                        setIsModalOpen(true);
                      }}
                   >
                     <Pencil size={14} />
                   </button>
                   <button className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-colors" onClick={() => { deleteContract.mutate(contract.id); }}>
                     <Trash2 size={14} />
                   </button>
                </div>
              </div>
              
              <div className="grid grid-cols-2 gap-4 mt-6 p-4 bg-slate-50 dark:bg-slate-800/50 rounded-2xl">
                <div>
                  <div className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">Ngày hiệu lực</div>
                  <div className="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">{formatDate(contract.start_date)}</div>
                </div>
                <div>
                  <div className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">Ngày hết hạn</div>
                  <div className="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">{contract.end_date ? formatDate(contract.end_date) : "Không thời hạn"}</div>
                </div>
              </div>
              <FileText className="absolute -right-6 -bottom-6 w-24 h-24 text-slate-100 dark:text-slate-800 opacity-50 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
            </div>
          ))
        )}
      </div>
      
      <ContractFormModal 
        editingContract={editingContract}
        employeeId={employeeId}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        wsId={wsId}
      />
    </div>
  );
}

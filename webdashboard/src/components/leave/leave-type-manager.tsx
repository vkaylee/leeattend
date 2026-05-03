import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Wallet, AlertTriangle, Pencil, Plus, Trash2 } from "lucide-react";
import React, { useState } from "react";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { fetchLeaveTypes, updateLeaveTypeConfig, createLeaveType, deleteLeaveType } from "@/lib/api/leaves";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface LeaveTypeManagerProps {
  wsId: string;
}

export function LeaveTypeManager({ wsId }: LeaveTypeManagerProps) {
  const toast = useToast();
  const queryClient = useQueryClient();
  
  const [editingType, setEditingType] = useState<LeaveType | null>(null);
  const [isAddingType, setIsAddingType] = useState(false);
  const [deletingType, setDeletingType] = useState<LeaveType | null>(null);

  const { data: leaveTypes = [], isLoading: isTypesLoading } = useQuery({
    queryKey: ["leave-types", wsId],
    queryFn: () => fetchLeaveTypes(wsId),
  });

  const createTypeMutation = useMutation({
    mutationFn: (data: { code: string; name: string; is_paid: boolean }) => createLeaveType(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success("Leave type created successfully");
      setIsAddingType(false);
    },
    onError: (error: Error) => { toast.error(error.message); },
  });

  const updateTypeMutation = useMutation({
    mutationFn: (data: { typeId: string; config: LeaveTypeConfig }) => updateLeaveTypeConfig(wsId, data.typeId, data.config),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success("Leave type updated successfully");
      setEditingType(null);
    },
    onError: () => { toast.error("Failed to update leave type"); },
  });

  const deleteTypeMutation = useMutation({
    mutationFn: (typeId: string) => deleteLeaveType(wsId, typeId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success("Leave type deleted successfully");
      setDeletingType(null);
    },
    onError: (error: Error) => {
      toast.error(error.message);
      setDeletingType(null);
    },
  });

  if (isTypesLoading) return <div className="py-12 text-center text-slate-400 dark:text-slate-500 animate-pulse">Loading leave types...</div>;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center px-1">
         <p className="text-xs font-medium text-slate-500 dark:text-slate-400 max-w-md italic">Define the categories of leave available in your organization (e.g., Annual Leave, Sick Leave).</p>
         <button className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100" onClick={() => { setIsAddingType(true); }}><Plus size={14} /> Add Leave Type</button>
      </div>
      
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {leaveTypes.map((type) => (
          <div className="relative flex flex-col justify-between rounded-[2rem] border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-7 shadow-sm hover:shadow-md transition-all group" key={type.id}>
            <div>
              <div className="flex items-center justify-between">
                <div className={cn("flex h-12 w-12 items-center justify-center rounded-2xl shadow-inner", type.is_paid ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800" : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border border-amber-100 dark:border-amber-800")}>
                  {type.is_paid ? <Wallet size={20} /> : <AlertTriangle size={20} />}
                </div>
                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button className="p-2 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:text-primary-400 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800" onClick={() => { setEditingType(type); }}><Pencil size={16} /></button>
                  <button className="p-2 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-800" onClick={() => { setDeletingType(type); }}><Trash2 size={16} /></button>
                </div>
              </div>
              <h3 className="mt-5 text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none uppercase">{type.name}</h3>
              <p className="mt-1 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">Code: {type.code}</p>
              <div className="mt-4 flex gap-2">
                 <span className={cn("px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border", type.is_paid ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-100 dark:border-amber-800")}>{type.is_paid ? "Paid" : "Unpaid"}</span>
              </div>
            </div>
          </div>
        ))}
      </div>

      {!!isAddingType && <AddLeaveTypeModal isSaving={createTypeMutation.isPending} onClose={() => { setIsAddingType(false); }} onSave={(data) => { createTypeMutation.mutate(data); }} />}
      {!!editingType && <EditLeaveTypeModal isSaving={updateTypeMutation.isPending} leaveType={editingType} onClose={() => { setEditingType(null); }} onSave={(config: LeaveTypeConfig) => { updateTypeMutation.mutate({ typeId: editingType.id, config }); }} />}
      { }
      {!!deletingType && (
        <Modal isOpen={true} onClose={() => { setDeletingType(null); }} title="Confirm Delete">
          <div className="p-8">
            <p className="text-sm">Are you sure you want to delete "{deletingType.name}"?</p>
            <div className="mt-8 flex justify-end gap-3"><button className="px-6 py-2 text-xs font-bold text-slate-400 dark:text-slate-500" onClick={() => { setDeletingType(null); }}>Cancel</button><button className="rounded-xl bg-rose-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase" onClick={() => { deleteTypeMutation.mutate(deletingType.id); }}>Delete</button></div>
          </div>
        </Modal>
      )}
    </div>
  );
}

interface AddLeaveTypeModalProps {
  isSaving: boolean;
 
  onClose: () => void;
  onSave: (data: { code: string; name: string; is_paid: boolean }) => void;
}

function AddLeaveTypeModal({ onClose, onSave, isSaving }: AddLeaveTypeModalProps) {
  const [formData, setFormData] = useState({ code: "", name: "", is_paid: true });
  return (
 
    <Modal isOpen={true} onClose={onClose} title="Add Leave Type">
      <form className="p-8 space-y-5" onSubmit={(e) => { e.preventDefault(); onSave(formData); }}>
        <input className="w-full rounded-2xl border p-4 text-sm" onChange={(e) => { setFormData({ ...formData, code: e.target.value.toUpperCase() }); }} placeholder="Code" required type="text" value={formData.code} />
        <input className="w-full rounded-2xl border p-4 text-sm" onChange={(e) => { setFormData({ ...formData, name: e.target.value }); }} placeholder="Name" required type="text" value={formData.name} />
        <div className="flex items-center gap-3"><input checked={formData.is_paid} id="paid-leave-input" onChange={(e) => { setFormData({ ...formData, is_paid: e.target.checked }); }} type="checkbox" /><label className="text-xs font-bold uppercase" htmlFor="paid-leave-input">Paid Leave?</label></div>
        <div className="flex justify-end gap-3 pt-6"><button onClick={onClose} type="button">Cancel</button><button className="rounded-xl bg-primary-600 px-8 py-3 text-xs font-bold text-white uppercase" disabled={isSaving} type="submit">Create</button></div>
      </form>
      { }
    </Modal>
  );
}

interface EditLeaveTypeModalProps {
  isSaving: boolean;
  leaveType: LeaveType;
  onClose: () => void;
  onSave: (config: LeaveTypeConfig) => void;
}

function EditLeaveTypeModal({ leaveType, onClose, onSave, isSaving }: EditLeaveTypeModalProps) {
  const [config, setConfig] = useState(leaveType.policy_config || { approval_required: true, requires_document: false, allow_negative_balance: false });
  return (
    <Modal isOpen={true} onClose={onClose} title={`Config: ${leaveType.name ?? ""}`}>
      <form className="p-8 space-y-4" onSubmit={(e) => { e.preventDefault(); onSave(config); }}>
        <div className="flex items-center gap-3"><input checked={config.approval_required} id="approval-req-input" onChange={(e) => { setConfig({ ...config, approval_required: e.target.checked }); }} type="checkbox" /><label className="text-xs font-bold uppercase" htmlFor="approval-req-input">Approval Required</label></div>
        <div className="flex justify-end gap-3 pt-6"><button onClick={onClose} type="button">Cancel</button><button className="rounded-xl bg-primary-600 px-8 py-3 text-xs font-bold text-white uppercase" disabled={isSaving} type="submit">Save</button></div>
      </form>
    </Modal>
  );
}

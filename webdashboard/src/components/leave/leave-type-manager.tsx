import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Wallet, AlertTriangle, Pencil, Plus, Trash2, Sparkles, Search, ChevronRight } from "lucide-react";
import React, { useState, useMemo } from "react";

import EmptyState from "@/components/ui/empty-state";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { fetchLeaveTypes, updateLeaveTypeConfig, createLeaveType, deleteLeaveType } from "@/lib/api/leaves";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { LEAVE_TEMPLATES, LeaveTemplate } from "./leave-templates";

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
      
      {leaveTypes.length > 0 ? (
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
      ) : (
        <EmptyState 
          action={
            <button 
              className="flex items-center gap-2 rounded-xl bg-primary-600 px-8 py-3 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-200 dark:shadow-none" 
              onClick={() => { setIsAddingType(true); }}
            >
              <Plus size={16} /> 
              Get Started
            </button>
          }
          description="Your workspace is fresh and clean. Start by creating leave types manually or use our professional templates."
          icon={<Sparkles className="text-primary-400" size={40} />}
          title="No Leave Types Configured"
        />
      )}

      {isAddingType ? <AddLeaveTypeModal isSaving={createTypeMutation.isPending} onClose={() => { setIsAddingType(false); }} onSave={(data) => { createTypeMutation.mutate(data); }} /> : null}
      {editingType ? <EditLeaveTypeModal isSaving={updateTypeMutation.isPending} leaveType={editingType} onClose={() => { setEditingType(null); }} onSave={(config: LeaveTypeConfig) => { updateTypeMutation.mutate({ typeId: editingType.id, config }); }} /> : null}
      { }
      {deletingType ? (
        <Modal isOpen={true} onClose={() => { setDeletingType(null); }} title="Confirm Delete">
          <div className="p-8">
            <p className="text-sm">Are you sure you want to delete &quot;{deletingType.name}&quot;?</p>
            <div className="mt-8 flex justify-end gap-3"><button className="px-6 py-2 text-xs font-bold text-slate-400 dark:text-slate-500" onClick={() => { setDeletingType(null); }}>Cancel</button><button className="rounded-xl bg-rose-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase" onClick={() => { deleteTypeMutation.mutate(deletingType.id); }}>Delete</button></div>
          </div>
        </Modal>
      ) : null}
    </div>
  );
}

interface AddLeaveTypeModalProps {
  isSaving: boolean;
 
  onClose: () => void;
  onSave: (data: { code: string; name: string; is_paid: boolean }) => void;
}

// eslint-disable-next-line complexity, max-lines-per-function
function AddLeaveTypeModal({ onClose, onSave, isSaving }: AddLeaveTypeModalProps) {
  const [step, setStep] = useState<'template' | 'form'>('template');
  const [formData, setFormData] = useState({ code: "", name: "", is_paid: true });
  const [search, setSearch] = useState("");
  const [activeCategory, setActiveCategory] = useState<string>("all");

  const handleApplyTemplate = (template: LeaveTemplate) => {
    setFormData({
      code: template.code,
      name: template.name,
      is_paid: template.is_paid
    });
    setStep('form');
  };

  const categories = [
    { id: "all", name: "All" },
    { id: "standard", name: "Standard" },
    { id: "wellness", name: "Wellness" },
    { id: "family", name: "Family" },
    { id: "professional", name: "Career" },
    { id: "special", name: "Special" },
  ];

  const filteredTemplates = useMemo(() => {
    return LEAVE_TEMPLATES.filter(t => {
      const matchesSearch = t.name.toLowerCase().includes(search.toLowerCase()) || 
                           t.code.toLowerCase().includes(search.toLowerCase());
      const matchesCategory = activeCategory === "all" || t.category === activeCategory;
      return matchesSearch && matchesCategory;
    });
  }, [search, activeCategory]);

  const getColorClasses = (color: string) => {
    const map: Record<string, string> = {
      blue: "bg-blue-50 dark:bg-blue-950/30 text-blue-600 border-blue-100 dark:border-blue-900/50",
      emerald: "bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 border-emerald-100 dark:border-emerald-900/50",
      rose: "bg-rose-50 dark:bg-rose-950/30 text-rose-600 border-rose-100 dark:border-rose-900/50",
      amber: "bg-amber-50 dark:bg-amber-950/30 text-amber-600 border-amber-100 dark:border-amber-900/50",
      indigo: "bg-indigo-50 dark:bg-indigo-950/30 text-indigo-600 border-indigo-100 dark:border-indigo-900/50",
      violet: "bg-violet-50 dark:bg-violet-950/30 text-violet-600 border-violet-100 dark:border-violet-900/50",
      pink: "bg-pink-50 dark:bg-pink-950/30 text-pink-600 border-pink-100 dark:border-pink-900/50",
      cyan: "bg-cyan-50 dark:bg-cyan-950/30 text-cyan-600 border-cyan-100 dark:border-cyan-900/50",
      teal: "bg-teal-50 dark:bg-teal-950/30 text-teal-600 border-teal-100 dark:border-teal-900/50",
      orange: "bg-orange-50 dark:bg-orange-950/30 text-orange-600 border-orange-100 dark:border-orange-900/50",
      slate: "bg-slate-50 dark:bg-slate-800/50 text-slate-600 border-slate-100 dark:border-slate-800",
      stone: "bg-stone-50 dark:bg-stone-900/50 text-stone-600 border-stone-100 dark:border-stone-800",
      brown: "bg-amber-100/50 dark:bg-amber-950/20 text-amber-800 dark:text-amber-400 border-amber-200 dark:border-amber-900",
      lime: "bg-lime-50 dark:bg-lime-950/30 text-lime-600 border-lime-100 dark:border-lime-900/50",
    };
    return map[color] || map.slate;
  };

  return (
    <Modal 
      isOpen={true} 
      maxWidth={step === 'template' ? "2xl" : "md"} 
      onClose={onClose}
      subtitle={step === 'template' ? "Start faster with a professional setup" : `Refining ${formData.name || 'New Leave Type'}`}
      title={step === 'template' ? "Choose a Template" : "Customize Leave Type"}
    >
      {step === 'template' ? (
        <div className="flex flex-col h-[65vh] max-h-[700px] -m-8">
          {/* Header with Search */}
          <div className="px-8 pt-6 pb-6 border-b border-slate-50 dark:border-slate-800 bg-white/50 dark:bg-slate-900/50 backdrop-blur-sm sticky top-0 z-10">
            <div className="flex items-center justify-between gap-4">
              <div className="relative flex-1">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                <input 
                  className="w-full bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 rounded-2xl py-3 pl-11 pr-4 text-xs focus:ring-2 focus:ring-primary-500/20 outline-none transition-all" 
                  onChange={(e) => { setSearch(e.target.value); }}
                  placeholder="Search templates (e.g. 'Annual', 'Sick')..."
                  type="text"
                  value={search}
                />
              </div>
              <button
                className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:underline whitespace-nowrap"
                onClick={() => { setStep('form'); }}
              >
                Skip to manual
              </button>
            </div>
            
            <div className="flex flex-wrap gap-2 mt-4 pb-2">
              {categories.map((cat) => (
                <button
                  className={cn(
                    "px-4 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest whitespace-nowrap transition-all",
                    activeCategory === cat.id 
                      ? "bg-primary-600 text-white shadow-lg shadow-primary-500/20" 
                      : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-700"
                  )}
                  key={cat.id}
                  onClick={() => { setActiveCategory(cat.id); }}
                >
                  {cat.name}
                </button>
              ))}
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
            {filteredTemplates.length > 0 ? (
              <div className="grid grid-cols-1 gap-4">
                {filteredTemplates.map((template) => {
                  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                  const Icon = template.icon;
                  return (
                    <button
                      className="group relative flex items-start gap-3 p-4 rounded-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/50 hover:border-primary-200 dark:hover:border-primary-800 hover:bg-primary-50/30 dark:hover:bg-primary-950/10 transition-all text-left"
                      key={template.id}
                      onClick={() => { handleApplyTemplate(template); }}
                    >
                      <div className={cn("h-10 w-10 shrink-0 rounded-xl flex items-center justify-center border transition-transform group-hover:scale-105", getColorClasses(template.color))}>
                        <Icon size={18} />
                      </div>
                      <div className="flex-1 min-w-0 pr-6">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="text-sm font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight truncate">{template.name}</h4>
                          <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 shrink-0">{template.code}</span>
                        </div>
                        <p className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed">{template.description}</p>
                        <div className="mt-2 flex items-center gap-2">
                          <span className={cn(
                            "px-2 py-0.5 rounded-md text-[9px] font-bold uppercase tracking-widest",
                            template.is_paid ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600" : "bg-amber-50 dark:bg-amber-950/50 text-amber-600"
                          )}>
                            {template.is_paid ? "Paid" : "Unpaid"}
                          </span>
                        </div>
                      </div>
                      <ChevronRight className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-700 group-hover:text-primary-400 transition-all group-hover:translate-x-1" size={16} />
                    </button>
                  );
                })}
              </div>
            ) : (
              <div className="h-full flex flex-col items-center justify-center py-12 text-center">
                <Search className="text-slate-200 dark:text-slate-800 mb-4" size={32} />
                <h4 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">No templates found</h4>
                <p className="text-xs text-slate-400 dark:text-slate-500 mt-1 max-w-[200px]">Try a different keyword or category</p>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="relative">
          <form className="space-y-6" onSubmit={(e) => { e.preventDefault(); onSave(formData); }}>
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest ml-1" htmlFor="type-code-input">Type Code</label>
              <input 
                className="w-full rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all outline-none"
                id="type-code-input" 
                onChange={(e) => { setFormData({ ...formData, code: e.target.value.toUpperCase() }); }} 
                placeholder="e.g. AL, SL, UL" 
                required 
                type="text" 
                value={formData.code} 
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest ml-1" htmlFor="display-name-input">Display Name</label>
              <input 
                className="w-full rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-4 text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all outline-none"
                id="display-name-input" 
                onChange={(e) => { setFormData({ ...formData, name: e.target.value }); }} 
                placeholder="e.g. Annual Leave" 
                required 
                type="text" 
                value={formData.name} 
              />
            </div>

            <div className="flex items-center justify-between p-5 rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800">
              <div className="flex items-center gap-3">
                <div className={cn("h-10 w-10 rounded-xl flex items-center justify-center transition-colors shadow-inner", formData.is_paid ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600" : "bg-amber-50 dark:bg-amber-950/50 text-amber-600")}>
                  {formData.is_paid ? <Wallet size={18} /> : <AlertTriangle size={18} />}
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" htmlFor="paid-leave-input">Paid Status</label>
                  <p className="text-[10px] text-slate-400 dark:text-slate-500">{formData.is_paid ? "Paid time off" : "Unpaid leave"}</p>
                </div>
              </div>
              <div className="relative inline-flex items-center cursor-pointer">
                <input 
                  checked={formData.is_paid} 
                  className="sr-only peer" 
                  id="paid-leave-input" 
                  onChange={(e) => { setFormData({ ...formData, is_paid: e.target.checked }); }} 
                  type="checkbox"
                />
                <div className="w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
              </div>
            </div>

            <div className="flex items-center justify-between pt-6 border-t border-slate-50 dark:border-slate-800">
              <button 
                className="flex items-center gap-2 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-primary-600 transition-colors uppercase tracking-widest" 
                onClick={() => { setStep('template'); }} 
                type="button"
              >
                ← Change Template
              </button>
              <div className="flex gap-3">
                <button 
                  className="px-6 py-3 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-slate-600 transition-colors uppercase tracking-widest" 
                  onClick={onClose} 
                  type="button"
                >
                  Cancel
                </button>
                <button 
                  className="rounded-2xl bg-primary-600 px-8 py-3.5 text-[10px] font-bold text-white uppercase tracking-[0.2em] shadow-lg shadow-primary-500/20 hover:bg-primary-700 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50" 
                  disabled={isSaving} 
                  type="submit"
                >
                  {isSaving ? "Creating..." : "Confirm & Create"}
                </button>
              </div>
            </div>
          </form>
        </div>
      )}
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
  const [config, setConfig] = useState(leaveType.policy_config ?? { 
    approval_required: true, 
    requires_document: false, 
    allow_negative_balance: false 
  });

  return (
    <Modal 
      isOpen={true} 
      maxWidth="md" 
      onClose={onClose} 
      subtitle={`Defining rules for ${leaveType.name}`}
      title="Leave Policy Configuration"
    >
      <form className="space-y-6" onSubmit={(e) => { e.preventDefault(); onSave(config); }}>
        <div className="rounded-[2rem] bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 overflow-hidden">
          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-primary-50 dark:bg-primary-950/50 text-primary-600 flex items-center justify-center shadow-inner shrink-0">
                <Search size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="approval-req-input">Approval Required</label>
                <p className="text-[10px] text-slate-400 dark:text-slate-500">Managers must approve requests</p>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input 
                checked={config.approval_required} 
                className="sr-only peer" 
                id="approval-req-input" 
                onChange={(e) => { setConfig({ ...config, approval_required: e.target.checked }); }} 
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 border-b border-slate-100 dark:border-slate-800">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-blue-50 dark:bg-blue-950/50 text-blue-600 flex items-center justify-center shadow-inner shrink-0">
                <Sparkles size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="doc-req-input">Requires Document</label>
                <p className="text-[10px] text-slate-400 dark:text-slate-500">Proof/Medical cert is mandatory</p>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input 
                checked={config.requires_document} 
                className="sr-only peer" 
                id="doc-req-input" 
                onChange={(e) => { setConfig({ ...config, requires_document: e.target.checked }); }} 
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4">
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-xl bg-amber-50 dark:bg-amber-950/50 text-amber-600 flex items-center justify-center shadow-inner shrink-0">
                <AlertTriangle size={16} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight block" htmlFor="negative-bal-input">Negative Balance</label>
                <p className="text-[10px] text-slate-400 dark:text-slate-500">Allow users to take leave beyond 0</p>
              </div>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input 
                checked={config.allow_negative_balance} 
                className="sr-only peer" 
                id="negative-bal-input" 
                onChange={(e) => { setConfig({ ...config, allow_negative_balance: e.target.checked }); }} 
                type="checkbox"
              />
              <div className="w-10 h-5.5 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4.5 after:w-4.5 after:transition-all peer-checked:bg-primary-600"></div>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-50 dark:border-slate-800">
          <button 
            className="px-6 py-3 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-slate-600 transition-colors uppercase tracking-widest" 
            onClick={onClose} 
            type="button"
          >
            Cancel
          </button>
          <button 
            className="rounded-2xl bg-primary-600 px-10 py-3.5 text-[10px] font-bold text-white uppercase tracking-[0.2em] shadow-lg shadow-primary-500/20 hover:bg-primary-700 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50" 
            disabled={isSaving} 
            type="submit"
          >
            {isSaving ? "Saving..." : "Save Policy"}
          </button>
        </div>
      </form>
    </Modal>
  );
}

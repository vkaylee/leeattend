import React, { useState } from "react";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, _CheckCircle2, _CircleDashed, Users, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";

import { fetchEmploymentPhases, createEmploymentPhase } from "@/lib/api/employees";
import { EmploymentPhaseDefinition } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  wsId: string;
}

export default function EmploymentPhaseManager({ wsId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: phases, isLoading } = useQuery({ queryKey: ["employment-phases", wsId], queryFn: () => fetchEmploymentPhases(wsId), enabled: !!wsId });
  const [formData, setFormData] = useState<Partial<EmploymentPhaseDefinition>>({ name: "", description: "", phase_type: "probation", default_duration_days: 60, is_active: true, display_order: 1 });

  const createMutation = useMutation({
    mutationFn: (data: Partial<EmploymentPhaseDefinition>) => createEmploymentPhase(wsId, data),
    onSuccess: () => { toast.success(t("common.success")); queryClient.invalidateQueries({ queryKey: ["employment-phases"] }); setIsModalOpen(false); setFormData({ name: "", description: "", phase_type: "probation", default_duration_days: 60, is_active: true, display_order: 1 }); },
  });

  const handleSubmit = (e: React.FormEvent) => { e.preventDefault(); createMutation.mutate(formData); };

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in duration-700">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30 px-12">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-indigo-50 dark:bg-indigo-950 flex items-center justify-center text-indigo-600 dark:text-indigo-300 shadow-inner border border-indigo-100 dark:border-indigo-800/50"><Users size={24} /></div>
          <div>
            <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none">Employment Phases</h2>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">Manage probation, intern and official phases</p>
          </div>
        </div>
        <button onClick={() => setIsModalOpen(true)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"><Plus size={18} /> {t("common.add").toUpperCase()}</button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : phases?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><Users size={32} className="text-slate-200 dark:text-slate-700" /></div>
            <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">No Employment Phases</p>
          </div>
        ) : (
          phases?.map((phase) => (
            <div key={phase.id} className="p-8 px-10 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group relative rounded-3xl m-2">
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-inner group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 group-hover:shadow-md transition-all uppercase">{phase.name.slice(0, 2)}</div>
                <div>
                  <p className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{phase.name}</p>
                  <p className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-2 opacity-70">{phase.phase_type}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-10">
                <div className="flex flex-col items-end gap-2">
                  <span className={cn("text-[9px] font-bold uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", phase.is_active ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>
                    {phase.is_active ? "ACTIVE" : "INACTIVE"}
                  </span>
                  <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-50 italic">{phase.default_duration_days} DAYS</span>
                </div>
                <button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90"><Trash2 size={18} /></button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title="Add Phase" subtitle="Create a new employment phase" icon={<Users size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="phase-name" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.name")}</label>
              <input id="phase-name" required type="text" placeholder="Probation" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" value={formData.name} onChange={e => setFormData({...formData, name: e.target.value})} />
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="phase-type" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Type</label>
              <select id="phase-type" required className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.phase_type} onChange={e => setFormData({...formData, phase_type: e.target.value as any})}>
                <option value="probation">Probation</option>
                <option value="internship">Internship</option>
                <option value="official">Official</option>
                <option value="contract">Contract</option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label htmlFor="phase-duration" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Default Duration (Days)</label>
              <input id="phase-duration" type="number" min="0" placeholder="60" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm" value={formData.default_duration_days || ""} onChange={e => setFormData({...formData, default_duration_days: parseInt(e.target.value) || 0})} />
            </div>
            <div className="space-y-3 px-1">
              <label htmlFor="phase-order" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">Display Order</label>
              <input id="phase-order" type="number" min="1" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm" value={formData.display_order || 1} onChange={e => setFormData({...formData, display_order: parseInt(e.target.value) || 1})} />
            </div>
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50 dark:border-slate-800">
            <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" disabled={createMutation.isPending} className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}

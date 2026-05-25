import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
/* eslint-disable max-lines-per-function */
import { Plus, Trash2, Users, Clock, Infinity as InfinityIcon } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
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
  const [formData, setFormData] = useState<Partial<EmploymentPhaseDefinition> & { default_duration_days?: number }>({ name: "", description: "", phase_type: "probation", default_duration_days: 60, is_active: true, display_order: 1 });

  const createMutation = useMutation({
    mutationFn: (data: Partial<EmploymentPhaseDefinition> & { rule_config?: { duration_days?: number } }) => createEmploymentPhase(wsId, data),
    onSuccess: () => { 
        toast.success(t("common.success")); 
        void queryClient.invalidateQueries({ queryKey: ["employment-phases"] }); 
        setIsModalOpen(false); 
        setFormData({ name: "", description: "", phase_type: "probation", default_duration_days: 60, is_active: true, display_order: 1 }); 
    },
    onError: (err: unknown) => {
        const error = err as Error;
        toast.error(error.message || t("common.error"));
    },
  });

  const handleSubmit = (e: React.SyntheticEvent) => { 
    e.preventDefault(); 
    createMutation.mutate({
      name: formData.name,
      description: formData.description,
      phase_type: formData.phase_type,
      display_order: formData.display_order,
      rule_config: { duration_days: formData.default_duration_days }
    }); 
  };

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in duration-700">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30 px-12">
        <div className="flex items-start gap-5">
          <div className="w-14 h-14 rounded-3xl bg-blue-50 dark:bg-blue-950/50 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-inner border border-blue-100 dark:border-blue-900 shrink-0">
            <Users size={28} />
          </div>
          <div className="max-w-xl">
            <Typography className="text-xl font-extrabold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none" variant="h2">Career Stages</Typography>
            <Typography className="text-[11px] font-medium text-slate-500 dark:text-slate-400 leading-relaxed mt-2" variant="p">
              Quản lý các cột mốc trong lộ trình phát triển của nhân sự (VD: Onboarding, Thử việc, Chính thức). 
              <br/>
              <span className="text-blue-600 dark:text-blue-400 font-bold bg-blue-50 dark:bg-blue-950/30 px-1 py-0.5 rounded mt-1 inline-block">💡 Mẹo:</span> Thứ tự sắp xếp sẽ quyết định vị trí hiển thị của Cột mốc trong các danh sách chọn.
            </Typography>
          </div>
        </div>
        <button className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={() => { setIsModalOpen(true); }}><Plus size={18} /> Thêm Cột mốc</button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : phases?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><Users className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">Chưa có cột mốc nào được định nghĩa</Typography>
          </div>
        ) : (
          phases?.map((phase) => (
            <div className="p-8 px-10 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group relative rounded-3xl m-2" key={phase.id}>
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-inner group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 group-hover:shadow-md transition-all uppercase">{phase.name.slice(0, 2)}</div>
                <div>
                  <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none" variant="p">{phase.name}</Typography>
                  <Typography className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-2 opacity-70" variant="p">{phase.phase_type}</Typography>
                </div>
              </div>
              
              <div className="flex items-center gap-10">
                <div className="flex flex-col items-end gap-2.5">
                  <div className="flex items-center gap-2">
                    {phase.rule_config?.duration_days && phase.rule_config.duration_days > 0 ? (
                      <span className="flex items-center gap-1.5 text-[10px] font-bold text-amber-600 dark:text-amber-500 bg-amber-50 dark:bg-amber-950/30 px-3 py-1 rounded-full border border-amber-100 dark:border-amber-900/50 shadow-sm uppercase tracking-widest">
                        <Clock size={12} /> {phase.rule_config.duration_days} ngày
                      </span>
                    ) : (
                      <span className="flex items-center gap-1.5 text-[10px] font-bold text-blue-600 dark:text-blue-500 bg-blue-50 dark:bg-blue-950/30 px-3 py-1 rounded-full border border-blue-100 dark:border-blue-900/50 shadow-sm uppercase tracking-widest">
                        <InfinityIcon size={12} /> Vô thời hạn
                      </span>
                    )}
                    <span className={cn("text-[9px] font-bold uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", phase.is_active ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>
                      {phase.is_active ? "ACTIVE" : "INACTIVE"}
                    </span>
                  </div>
                  <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] flex items-center">
                    Thứ tự hiển thị: <span className="text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded ml-1 font-mono">#{phase.display_order}</span>
                  </span>
                </div>
                <button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90"><Trash2 size={18} /></button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal icon={<Users size={24} />} isOpen={isModalOpen} maxWidth="lg" onClose={() => { setIsModalOpen(false); }} subtitle="Tạo một Cột mốc mới cho lộ trình của nhân viên" title="Thêm Cột mốc">
        <form className="space-y-10 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="phase-name">Tên Cột mốc</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="phase-name" onChange={e => { setFormData({...formData, name: e.target.value}); }} placeholder="VD: Thử việc, Lên Senior..." required type="text" value={formData.name} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="phase-type">Type</label>
              { }
              <select className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm" id="phase-type" onChange={e => { setFormData({...formData, phase_type: e.target.value as "probation" | "internship" | "official" | "contract"}); }} required value={formData.phase_type}>
                <option value="probation">Probation</option>
                <option value="internship">Internship</option>
                <option value="official">Official</option>
                <option value="contract">Contract</option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="phase-duration">Default Duration (Days)</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm" id="phase-duration" min="0" onChange={e => { setFormData({...formData, default_duration_days: parseInt(e.target.value) || 0}); }} placeholder="60" type="number" value={formData.default_duration_days ?? ""} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center justify-between" htmlFor="phase-order">
                <span>Thứ tự Sắp xếp</span>
                <span className="text-[8px] bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-400 px-2 py-0.5 rounded-full normal-case font-medium">Số càng nhỏ, hiển thị càng lên trên</span>
              </label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm" id="phase-order" min="1" onChange={e => { setFormData({...formData, display_order: parseInt(e.target.value) || 1}); }} type="number" value={formData.display_order ?? 1} />
            </div>
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50 dark:border-slate-800">
            <button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95" onClick={() => { setIsModalOpen(false); }} type="button">{t("common.cancel").toUpperCase()}</button>
            <button className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={createMutation.isPending} type="submit">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}

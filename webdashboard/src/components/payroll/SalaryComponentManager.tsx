import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, List, Coins, ShieldCheck, Percent, Trash2 } from "lucide-react";
import { fetchSalaryComponents, createSalaryComponent } from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";
import { useTranslation } from "react-i18next";
import { SalaryComponentDefinition } from "@/lib/api/types";
import Modal from "@/components/ui/Modal";

interface Props {
  wsId: string;
}

export default function SalaryComponentManager({ wsId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: components, isLoading } = useQuery({ queryKey: ["salary-components", wsId], queryFn: () => fetchSalaryComponents(wsId), enabled: !!wsId });
  const [formData, setFormData] = useState<Partial<SalaryComponentDefinition>>({ name: "", code: "", is_base_for_ot: true, is_taxable: true });

  const createMutation = useMutation({
    mutationFn: (data: Partial<SalaryComponentDefinition>) => createSalaryComponent(wsId, data),
    onSuccess: () => { toast.success(t("common.success")); queryClient.invalidateQueries({ queryKey: ["salary-components"] }); setIsModalOpen(false); setFormData({ name: "", code: "", is_base_for_ot: true, is_taxable: true }); },
  });

  const handleSubmit = (e: React.FormEvent) => { e.preventDefault(); createMutation.mutate(formData); };

  return (
    <div className="bg-white rounded-[2.5rem] border border-slate-100 shadow-sm overflow-hidden animate-in fade-in duration-700">
      <div className="p-10 border-b border-slate-50 flex items-center justify-between bg-slate-50/30 px-12">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-amber-50 flex items-center justify-center text-amber-600 shadow-inner border border-amber-100/50"><Coins size={24} /></div>
          <div>
            <h2 className="text-xl font-black text-slate-800 tracking-tight uppercase leading-none">{t("payroll.components.title")}</h2>
            <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("payroll.components.subtitle")}</p>
          </div>
        </div>
        <button onClick={() => setIsModalOpen(true)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95"><Plus size={18} /> {t("common.add").toUpperCase()}</button>
      </div>

      <div className="divide-y divide-slate-50 px-2">
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : components?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100"><Coins size={32} className="text-slate-200" /></div>
            <p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("payroll.components.no_components")}</p>
          </div>
        ) : (
          components?.map((comp) => (
            <div key={comp.id} className="p-8 px-10 flex items-center justify-between hover:bg-slate-50/50 transition-all group relative rounded-3xl m-2">
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 rounded-2xl bg-slate-50 border border-slate-100 flex items-center justify-center text-slate-400 font-black text-lg shadow-inner group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100 group-hover:shadow-md transition-all uppercase">{comp.code.slice(0, 2)}</div>
                <div>
                  <p className="font-black text-slate-900 text-sm tracking-tight uppercase leading-none">{comp.name}</p>
                  <p className="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-widest mt-2 opacity-70">{comp.code}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-10">
                <div className="flex flex-col items-end gap-2">
                  <span className={cn("text-[9px] font-black uppercase tracking-widest px-3 py-1 rounded-full border shadow-sm", comp.is_base_for_ot ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-50 text-slate-400 border-slate-100 opacity-60')}>
                    {comp.is_base_for_ot ? t("payroll.components.use_for_ot").toUpperCase() : t("payroll.components.no_ot").toUpperCase()}
                  </span>
                  <span className="text-[8px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-50 italic">{comp.is_taxable ? t("payroll.components.taxable").toUpperCase() : t("payroll.components.tax_exempt").toUpperCase()}</span>
                </div>
                <button className="p-2.5 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100 active:scale-90"><Trash2 size={18} /></button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={t("payroll.components.add_new")} subtitle={t("payroll.components.form_subtitle")} icon={<Coins size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.name")}</label>
              <input required type="text" placeholder={t("payroll.components.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.name} onChange={e => setFormData({...formData, name: e.target.value})} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("payroll.components.code_label")}</label>
              <input required type="text" placeholder="MEAL" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-mono font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight shadow-sm" value={formData.code} onChange={e => setFormData({...formData, code: e.target.value.toUpperCase()})} />
            </div>
          </div>

          <div className="space-y-5">
            {[
              { key: "is_base_for_ot", label: "ot_base_label", desc: "ot_base_desc", icon: ShieldCheck },
              { key: "is_taxable", label: "taxable_label", desc: "taxable_desc", icon: Percent }
            ].map(field => (
              <label key={field.key} className={cn("flex items-center gap-6 p-8 rounded-[2rem] cursor-pointer transition-all group/check border-2 shadow-sm active:scale-[0.98]", 
                (formData as any)[field.key] ? "bg-white border-primary-100 shadow-xl shadow-primary-900/5" : "bg-slate-50/50 border-transparent hover:bg-white hover:border-slate-100 shadow-inner"
              )}>
                <input type="checkbox" className="w-7 h-7 rounded-xl text-primary-600 focus:ring-primary-500 transition-all border-slate-200 shadow-inner cursor-pointer" checked={(formData as any)[field.key]} onChange={e => setFormData({...formData, [field.key]: e.target.checked})} />
                <div>
                  <p className="font-black text-slate-800 text-sm uppercase tracking-tight group-hover/check:text-primary-600 transition-colors flex items-center gap-2"><field.icon size={14} className="opacity-50" /> {t(`payroll.components.${field.label}`)}</p>
                  <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mt-1.5 opacity-70 leading-relaxed max-w-sm">{t(`payroll.components.${field.desc}`)}</p>
                </div>
              </label>
            ))}
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50">
            <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" disabled={createMutation.isPending} className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
    </div>
  );
}

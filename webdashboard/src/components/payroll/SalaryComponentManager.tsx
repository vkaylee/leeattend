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
    onSuccess: () => { toast.success(t("common.success")); queryClient.invalidateQueries({ queryKey: ["salary-components", wsId] }); setIsModalOpen(false); setFormData({ name: "", code: "", is_base_for_ot: true, is_taxable: true }); },
  });

  const handleSubmit = (e: React.FormEvent) => { e.preventDefault(); createMutation.mutate(formData); };

  return (
    <div className="bg-white rounded-[2rem] border border-slate-100 shadow-sm overflow-hidden animate-in fade-in duration-500">
      <div className="p-8 border-b border-slate-50 flex items-center justify-between bg-slate-50/30">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center text-amber-600 shadow-inner border border-amber-100/50"><Coins size={20} /></div>
          <div><h2 className="text-lg font-black text-slate-800 tracking-tight uppercase">{t("payroll.components.title")}</h2><p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-0.5 opacity-70">{t("payroll.components.subtitle", { defaultValue: "Manage earnings and deductions" })}</p></div>
        </div>
        <button onClick={() => setIsModalOpen(true)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-black text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95"><Plus size={14} /> {t("common.add")}</button>
      </div>

      <div className="divide-y divide-slate-50">
        {isLoading ? (
          <div className="p-16 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>
        ) : components?.length === 0 ? (
          <div className="p-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("payroll.components.no_components")}</div>
        ) : (
          components?.map((comp) => (
            <div key={comp.id} className="p-8 flex items-center justify-between hover:bg-slate-50/50 transition-all group relative">
              <div className="flex items-center gap-5">
                <div className="w-12 h-12 rounded-2xl bg-slate-50 border border-slate-100 flex items-center justify-center text-slate-400 font-black shadow-inner group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100 group-hover:shadow-md transition-all">{comp.code.slice(0, 2).toUpperCase()}</div>
                <div>
                  <p className="font-black text-slate-900 text-sm tracking-tight uppercase">{comp.name}</p>
                  <p className="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-tighter mt-0.5">{comp.code}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-8">
                <div className="flex flex-col items-end gap-1.5">
                  <span className={cn("text-[9px] font-black uppercase tracking-widest px-2.5 py-1 rounded-full border shadow-sm", comp.is_base_for_ot ? 'bg-emerald-50 text-emerald-600 border-emerald-100' : 'bg-slate-100 text-slate-400 border-slate-200')}>
                    {comp.is_base_for_ot ? t("payroll.components.use_for_ot") : t("payroll.components.no_ot")}
                  </span>
                  <span className="text-[8px] font-black text-slate-400 uppercase tracking-[0.15em] opacity-60 italic">{comp.is_taxable ? t("payroll.components.taxable") : t("payroll.components.tax_exempt")}</span>
                </div>
                <button className="p-2.5 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={t("payroll.components.add_new")} subtitle={t("payroll.components.form_subtitle", { defaultValue: "Define a new component for payroll calculation" })} icon={<Coins size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.name")}</label>
              <input required type="text" placeholder={t("payroll.components.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight" value={formData.name} onChange={e => setFormData({...formData, name: e.target.value})} />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("payroll.components.code_label")}</label>
              <input required type="text" placeholder="MEAL" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-mono font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight" value={formData.code} onChange={e => setFormData({...formData, code: e.target.value.toUpperCase()})} />
            </div>
          </div>

          <div className="space-y-4">
            {[
              { key: "is_base_for_ot", label: "ot_base_label", desc: "ot_base_desc" },
              { key: "is_taxable", label: "taxable_label", desc: "taxable_desc" }
            ].map(field => (
              <label key={field.key} className="flex items-center gap-5 p-6 bg-slate-50 rounded-[1.5rem] cursor-pointer hover:bg-white border-2 border-transparent hover:border-primary-100 shadow-inner hover:shadow-xl hover:shadow-primary-100/10 transition-all group/check">
                <input type="checkbox" className="w-6 h-6 rounded-xl text-primary-600 focus:ring-primary-500 transition-all border-slate-200" checked={(formData as any)[field.key]} onChange={e => setFormData({...formData, [field.key]: e.target.checked})} />
                <div>
                  <p className="font-black text-slate-800 text-xs uppercase tracking-tight group-hover/check:text-primary-600 transition-colors">{t(`payroll.components.${field.label}`)}</p>
                  <p className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mt-1 opacity-70 leading-relaxed">{t(`payroll.components.${field.desc}`)}</p>
                </div>
              </label>
            ))}
          </div>

          <div className="flex gap-4 pt-2">
            <button type="button" onClick={() => setIsModalOpen(false)} className="flex-1 px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button type="submit" disabled={createMutation.isPending} className="flex-1 bg-primary-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save")}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
    </div>
  );
}

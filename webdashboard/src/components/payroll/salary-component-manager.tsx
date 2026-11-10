import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Coins, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchSalaryComponents, createSalaryComponent, fetchSalaryBaseGroups } from "@/lib/api/payroll";
import { CreateSalaryComponentRequest } from "@/lib/api/types";
import { cn } from "@/lib/utils";

export default function SalaryComponentManager() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: components, isLoading } = useQuery({ queryKey: ["salary-components"], queryFn: () => fetchSalaryComponents() });
  const { data: baseGroupsData } = useQuery({ queryKey: ["salary-base-groups"], queryFn: () => fetchSalaryBaseGroups() });
  const [formData, setFormData] = useState<CreateSalaryComponentRequest>({ name: "", code: "", base_group_ids: [] });

  const createMutation = useMutation({
    mutationFn: (data: CreateSalaryComponentRequest) => createSalaryComponent(data),
    onSuccess: () => { toast.success(t("common.success")); void queryClient.invalidateQueries({ queryKey: ["salary-components"] }); void queryClient.invalidateQueries({ queryKey: ["salary-base-groups"] }); setIsModalOpen(false); setFormData({ name: "", code: "", base_group_ids: [] }); },
  });

  const handleSubmit = (e: React.SyntheticEvent) => { e.preventDefault(); createMutation.mutate(formData); };

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in duration-700">
      <div className="p-10 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30 px-12">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-amber-50 dark:bg-amber-950 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-inner border border-amber-100 dark:border-amber-800/50"><Coins size={24} /></div>
          <div>
            <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none" variant="h2">{t("payroll.components.title")}</Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70" variant="p">{t("payroll.components.subtitle")}</Typography>
          </div>
        </div>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={() => { setIsModalOpen(true); }} variant="ghost"><Plus size={18} /> {t("common.add").toUpperCase()}</Button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : components?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><Coins className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("payroll.components.no_components")}</Typography>
          </div>
        ) : (
          components?.map((comp) => (
            <div className="p-8 px-10 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group relative rounded-3xl m-2" key={comp.id}>
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 rounded-2xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-inner group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 group-hover:shadow-md transition-all uppercase">{comp.code.slice(0, 2)}</div>
                <div>
                  <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none" variant="p">{comp.name}</Typography>
                  <Typography className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-2 opacity-70" variant="p">{comp.code}</Typography>
                </div>
              </div>
              
              <div className="flex items-center gap-10">
                <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90" variant="ghost"><Trash2 size={18} /></Button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal icon={<Coins size={24} />} isOpen={isModalOpen} maxWidth="lg" onClose={() => { setIsModalOpen(false); }} subtitle={t("payroll.components.form_subtitle")} title={t("payroll.components.add_new")}>
        <form className="space-y-10 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="comp-name">{t("common.name")}</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="comp-name" onChange={e => { setFormData({...formData, name: e.target.value}); }} placeholder={t("payroll.components.name_placeholder")} required type="text" value={formData.name} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="comp-code">{t("payroll.components.code_label")}</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-mono font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all tracking-tight shadow-sm" id="comp-code" onChange={e => { setFormData({...formData, code: e.target.value.toUpperCase()}); }} placeholder="MEAL" required type="text" value={formData.code} />
            </div>
          </div>

          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.components.assign_to_bases")}</label>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {baseGroupsData?.groups.filter(g => g.slug !== "gross").map(group => {
                const isSelected = formData.base_group_ids?.includes(group.id);
                return (
                  <label aria-label={group.name} className={cn("flex items-center gap-4 p-4 rounded-2xl cursor-pointer transition-all group/check border-2 shadow-sm active:scale-[0.98]", isSelected ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800 shadow-md" : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:bg-white dark:hover:bg-slate-900 shadow-inner")} htmlFor={`group-${group.id}`} key={group.id}>
                    <input 
                      checked={isSelected} 
                      className="w-5 h-5 rounded-lg text-primary-600 dark:text-primary-400 focus:ring-primary-500 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer"
                      id={`group-${group.id}`}
                      onChange={(e) => {
                        const current = formData.base_group_ids ?? [];
                        setFormData({
                          ...formData,
                          base_group_ids: e.target.checked ? [...current, group.id] : current.filter(id => id !== group.id)
                        });
                      }}
                      type="checkbox"
                    />
                    <span className="flex flex-col">
                      <span className="font-bold text-slate-800 dark:text-slate-100 text-sm">{group.name}</span>
                      <span className="text-[10px] text-slate-500 dark:text-slate-400 mt-0.5 line-clamp-1">{group.description}</span>
                    </span>
                  </label>
                );
              })}
            </div>
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50 dark:border-slate-800">
            <Button className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95" onClick={() => { setIsModalOpen(false); }} type="button" variant="primary">{t("common.cancel").toUpperCase()}</Button>
            <Button className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50" disabled={createMutation.isPending} type="submit" variant="primary">{createMutation.isPending ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}

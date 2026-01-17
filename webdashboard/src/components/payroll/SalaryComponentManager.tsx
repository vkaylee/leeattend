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

  const { data: components, isLoading } = useQuery({
    queryKey: ["salary-components", wsId],
    queryFn: () => fetchSalaryComponents(wsId),
    enabled: !!wsId,
  });

  const [formData, setFormData] = useState<Partial<SalaryComponentDefinition>>({
    name: "",
    code: "",
    is_base_for_ot: true,
    is_taxable: true,
  });

  const createMutation = useMutation({
    mutationFn: (data: Partial<SalaryComponentDefinition>) => createSalaryComponent(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["salary-components", wsId] });
      setIsModalOpen(false);
      setFormData({ name: "", code: "", is_base_for_ot: true, is_taxable: true });
    },
    onError: (e: any) => {
      toast.error(e.message || t("common.error"));
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate(formData);
  };

  return (
    <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
      <div className="p-6 border-b border-slate-50 flex items-center justify-between">
        <h2 className="text-lg font-bold flex items-center gap-2 text-slate-800">
          <Coins className="w-5 h-5 text-amber-500" />
          {t("payroll.components.title", "Danh mục thành phần lương")}
        </h2>
        <button
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-2 bg-slate-900 text-white px-4 py-2 rounded-xl font-bold hover:bg-slate-800 transition-all text-sm"
        >
          <Plus className="w-4 h-4" />
          {t("common.add")}
        </button>
      </div>

      <div className="divide-y divide-slate-50">
        {isLoading ? (
          <div className="p-12 text-center text-slate-400 italic">{t("common.loading")}</div>
        ) : components?.length === 0 ? (
          <div className="p-12 text-center text-slate-400 italic">Chưa có thành phần lương nào</div>
        ) : (
          components?.map((comp) => (
            <div key={comp.id} className="p-6 flex items-center justify-between hover:bg-slate-50/50 transition-colors">
              <div className="flex items-center gap-4">
                <div className="w-10 h-10 rounded-xl bg-amber-50 text-amber-600 flex items-center justify-center font-bold">
                  {comp.code[0]}
                </div>
                <div>
                  <p className="font-bold text-slate-900">{comp.name}</p>
                  <p className="text-xs font-mono text-slate-400">{comp.code}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-6">
                <div className="flex flex-col items-end">
                  <span className={`text-[10px] font-black uppercase tracking-widest px-2 py-0.5 rounded-full ${
                    comp.is_base_for_ot ? 'bg-green-50 text-green-600 border border-green-100' : 'bg-slate-100 text-slate-400'
                  }`}>
                    {comp.is_base_for_ot ? "Dùng tính OT" : "Không tính OT"}
                  </span>
                  <span className="text-[10px] text-slate-400 mt-1 italic">
                    {comp.is_taxable ? "Có tính thuế" : "Miễn thuế"}
                  </span>
                </div>
                
                <button className="p-2 text-slate-300 hover:text-rose-600 transition-colors">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={t("payroll.components.add_new", "Thêm thành phần lương")}
      >
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.name")}</label>
              <input
                required
                type="text"
                placeholder="Ví dụ: Phụ cấp ăn trưa"
                className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-primary-500 transition-all"
                value={formData.name}
                onChange={e => setFormData({...formData, name: e.target.value})}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">Mã (Code)</label>
              <input
                required
                type="text"
                placeholder="MEAL"
                className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-primary-500 transition-all font-mono uppercase"
                value={formData.code}
                onChange={e => setFormData({...formData, code: e.target.value.toUpperCase()})}
              />
            </div>
          </div>

          <div className="space-y-4 pt-2">
            <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-2xl cursor-pointer hover:bg-slate-100 transition-colors">
              <input
                type="checkbox"
                className="w-5 h-5 rounded-lg text-primary-600 focus:ring-primary-500"
                checked={formData.is_base_for_ot}
                onChange={e => setFormData({...formData, is_base_for_ot: e.target.checked})}
              />
              <div>
                <p className="font-bold text-slate-800 text-sm">Dùng làm gốc tính lương OT</p>
                <p className="text-xs text-slate-500">Khi tính OT, hệ thống sẽ cộng dồn các khoản này để ra lương giờ.</p>
              </div>
            </label>

            <label className="flex items-center gap-3 p-4 bg-slate-50 rounded-2xl cursor-pointer hover:bg-slate-100 transition-colors">
              <input
                type="checkbox"
                className="w-5 h-5 rounded-lg text-primary-600 focus:ring-primary-500"
                checked={formData.is_taxable}
                onChange={e => setFormData({...formData, is_taxable: e.target.checked})}
              />
              <div>
                <p className="font-bold text-slate-800 text-sm">Khoản thu nhập tính thuế</p>
                <p className="text-xs text-slate-500">Khoản này sẽ được tính vào tổng thu nhập chịu thuế TNCN.</p>
              </div>
            </label>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={() => setIsModalOpen(false)}
              className="flex-1 px-6 py-3 rounded-xl font-bold text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              type="submit"
              disabled={createMutation.isPending}
              className="flex-1 bg-slate-900 text-white px-6 py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200 disabled:opacity-50"
            >
              {createMutation.isPending ? t("common.saving") : t("common.save")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}

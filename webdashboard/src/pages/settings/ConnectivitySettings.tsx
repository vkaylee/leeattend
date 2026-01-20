import { useState } from "react";
import { useTranslation } from "react-i18next";
import { HardDrive, Share2 } from "lucide-react";
import AgentsPage from "@/pages/AgentsPage";
import DevicesPage from "@/pages/DevicesPage";

type TabType = "agents" | "devices";

export default function ConnectivitySettings() {

  const { t } = useTranslation();

  const [activeTab, setActiveTab] = useState<TabType>("agents");



  return (

    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">

        <div className="flex items-center gap-4">

          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Share2 size={24} /></div>

          <div>

            <h1 className="text-3xl font-bold tracking-tight text-slate-900 leading-tight">{t("common.settings.categories.connectivity")}</h1>

            <p className="text-slate-500 mt-1 font-medium">{t("common.settings.categories.connectivity_desc")}</p>

          </div>

        </div>

      </div>



      <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">

        <button onClick={() => setActiveTab("agents")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'agents' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Share2 size={14} /> {t("common.settings.tabs.agents")}</button>

        <button onClick={() => setActiveTab("devices")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'devices' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><HardDrive size={14} /> {t("common.settings.tabs.devices")}</button>

      </div>



      <div className="animate-in slide-in-from-bottom-4 duration-500">

        {activeTab === "agents" && (

          <div className="bg-white rounded-[2.5rem] border border-slate-100 overflow-hidden shadow-sm p-4"><AgentsPage /></div>

        )}



        {activeTab === "devices" && (

          <div className="bg-white rounded-[2.5rem] border border-slate-100 overflow-hidden shadow-sm p-4"><DevicesPage /></div>

        )}

      </div>

    </div>

  );

}

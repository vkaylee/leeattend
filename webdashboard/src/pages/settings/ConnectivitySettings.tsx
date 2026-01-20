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
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
          {t("common.settings.categories.connectivity")}
        </h1>
        <p className="text-slate-500 text-sm mt-1">
          {t("common.settings.categories.connectivity_desc")}
        </p>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button
          onClick={() => setActiveTab("agents")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "agents"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Share2 size={14} /> {t("common.settings.tabs.agents", "Agents")}
        </button>
        <button
          onClick={() => setActiveTab("devices")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "devices"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <HardDrive size={14} /> {t("common.settings.tabs.devices", "Devices")}
        </button>
      </div>

      <div className="animate-in fade-in duration-500">
        {activeTab === "agents" && (
          <div className="bg-white rounded-[32px] border border-slate-100 overflow-hidden shadow-sm">
            <div className="p-1">
              <AgentsPage />
            </div>
          </div>
        )}

        {activeTab === "devices" && (
          <div className="bg-white rounded-[32px] border border-slate-100 overflow-hidden shadow-sm">
            <div className="p-1">
              <DevicesPage />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
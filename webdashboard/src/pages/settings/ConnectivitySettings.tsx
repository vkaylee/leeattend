import { useTranslation } from "react-i18next";
import AgentsPage from "@/pages/AgentsPage";
import DevicesPage from "@/pages/DevicesPage";

export default function ConnectivitySettings() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
          {t("common.settings.categories.connectivity")}
        </h1>
        <p className="text-slate-500 text-sm mt-1">
          {t("common.settings.categories.connectivity_desc")}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-8">
        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
           <div className="p-1">
              <AgentsPage />
           </div>
        </div>

        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
           <div className="p-1">
              <DevicesPage />
           </div>
        </div>
      </div>
    </div>
  );
}
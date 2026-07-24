import { Database } from "lucide-react";
import React from "react";

import { LABEL_BASE } from "@/components/ui/form-field";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Typography } from "@/components/ui/typography";

interface DatabaseConfigProps {
  databaseUrl: string;
  dbMode: "auto" | "byod";
  isSubmitting: boolean;
  setDatabaseUrl: (url: string) => void;
  setDbMode: (mode: "auto" | "byod") => void;
  t: (key: string) => string;
  validationError: string | null;
}

export function DatabaseConfig({
  databaseUrl,
  dbMode,
  isSubmitting,
  setDatabaseUrl,
  setDbMode,
  t,
  validationError,
}: DatabaseConfigProps) {
  return (
    <div className="space-y-2 px-1">
      <label className={LABEL_BASE}>
        {t("onboarding.db_mode_label")}
      </label>

      <Tabs onValueChange={(val) => { setDbMode(val as "auto" | "byod"); }} value={dbMode}>
        <TabsList className="grid grid-cols-2 p-1 bg-slate-100 dark:bg-slate-800/50 rounded-2xl border-none">
          <TabsTrigger
            className={`py-3 text-xs font-bold rounded-xl transition-all border-none ${
              dbMode === "auto"
                ? "bg-white dark:bg-slate-700 text-slate-800 dark:text-white shadow-sm"
                : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            value="auto"
          >
            {t("onboarding.db_mode_auto")}
          </TabsTrigger>
          <TabsTrigger
            className={`py-3 text-xs font-bold rounded-xl transition-all border-none ${
              dbMode === "byod"
                ? "bg-white dark:bg-slate-700 text-slate-800 dark:text-white shadow-sm"
                : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            value="byod"
          >
            {t("onboarding.db_mode_byod")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="auto">
          <div className="p-4 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border border-slate-100 dark:border-slate-800/50">
            <Typography className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed" variant="p">
              {t("onboarding.db_mode_auto_desc")}
            </Typography>
          </div>
        </TabsContent>

        <TabsContent value="byod">
          <div className="space-y-3 p-4 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border border-slate-100 dark:border-slate-800/50">
            <Typography className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed mb-1" variant="p">
              {t("onboarding.db_mode_byod_desc")}
            </Typography>

            <div className="space-y-1">
              <div className="relative flex items-center">
                <Database className="absolute left-4 text-slate-400 dark:text-slate-500" size={14} />
                <input
                  className={`w-full pl-11 pr-6 py-3.5 bg-white dark:bg-slate-800 border ${
                    validationError
                      ? "border-rose-400 dark:border-rose-800 focus:ring-rose-500/10"
                      : "border-slate-200 dark:border-slate-700 focus:ring-primary-500/10"
                  } rounded-xl text-xs focus:outline-none focus:ring-4 focus:bg-white dark:focus:bg-slate-900 transition-all font-mono tracking-tight shadow-sm`}
                  disabled={isSubmitting}
                  onChange={(e) => { setDatabaseUrl(e.target.value); }}
                  placeholder={t("onboarding.db_placeholder")}
                  required={dbMode === "byod"}
                  type="text"
                  value={databaseUrl}
                />
              </div>
              {validationError ? (
                <Typography className="text-[10px] text-rose-500 font-bold ml-1 tracking-tight" variant="p">
                  {validationError}
                </Typography>
              ) : (
                <Typography className="text-[9px] text-slate-400 dark:text-slate-500 ml-1 tracking-wide" variant="p">
                  {t("common.database_url_hint")}
                </Typography>
              )}
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

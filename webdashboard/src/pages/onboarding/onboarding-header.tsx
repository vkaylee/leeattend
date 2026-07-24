import { LogOut } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";

interface OnboardingHeaderProps {
  onLogout: () => void;
  t: (key: string) => string;
}

export function OnboardingHeader({ onLogout, t }: OnboardingHeaderProps) {
  return (
    <>
      {/* Top Right Actions */}
      <div className="absolute top-6 right-6 z-40">
        <Button className="flex items-center gap-2 px-4 py-2 rounded-full text-xs font-bold tracking-widest uppercase text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50 transition-all duration-300"
          onClick={onLogout}
          variant="ghost"
        >
          {t("common.logout")}
          <LogOut size={14} />
        </Button>
      </div>

      {/* Background Blobs */}
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/20 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-blue-100/30 dark:bg-blue-900/10 rounded-full blur-[140px] animate-pulse duration-[10s]" />
    </>
  );
}

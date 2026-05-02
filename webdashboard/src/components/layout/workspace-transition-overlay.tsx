import { Building2 } from "lucide-react";
import React, { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import { Workspace } from "@/lib/api";

interface Props {
  workspace: Workspace;
}

export default function WorkspaceTransitionOverlay({ workspace }: Props) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);

  useEffect(() => {
    // Sequence the text changes to show progress
    const t1 = setTimeout(() => { setStep(1); }, 800);
    const t2 = setTimeout(() => { setStep(2); }, 1800);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, []);

  const getStepText = () => {
    switch (step) {
      case 0:
        return t("common.switching_workspace", "Switching workspace...");
      case 1:
        return t("common.syncing_data", "Syncing data...");
      case 2:
        return t("common.preparing_environment", "Preparing environment...");
      default:
        return "";
    }
  };

  const overlayContent = (
    <div className="fixed inset-0 z-[99999] flex flex-col items-center justify-center bg-slate-900/60 dark:bg-slate-950/80 backdrop-blur-2xl animate-in fade-in duration-500">
      <div className="relative flex flex-col items-center max-w-sm w-full p-10 rounded-[2rem] bg-white/10 dark:bg-slate-900/60 border border-white/20 dark:border-white/10 shadow-2xl overflow-hidden backdrop-blur-md">
        
        {/* Animated background glow */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-48 h-48 bg-primary-500/20 rounded-full blur-[60px] animate-pulse" />

        {/* Brand Icon or Building Icon */}
        <div className="relative mb-8 bg-white/10 dark:bg-black/20 p-5 rounded-3xl border border-white/20 dark:border-white/10 shadow-inner">
          <Building2 className="text-white animate-pulse duration-1000" size={48} strokeWidth={1.5} />
        </div>

        {/* Text Details */}
        <div className="relative w-full text-center space-y-3">
          <h2 className="text-2xl font-bold text-white tracking-tight">
            {workspace.name}
          </h2>
          <div className="h-6 flex items-center justify-center overflow-hidden">
             <p 
               className="text-sm font-medium text-white/70 animate-in slide-in-from-bottom-4 fade-in duration-300" 
               key={step}
             >
               {getStepText()}
             </p>
          </div>
        </div>

        {/* Progress Line */}
        <div className="relative w-full h-1 mt-10 bg-white/10 rounded-full overflow-hidden">
          <div className="h-full bg-white dark:bg-primary-500 rounded-full progress-bar-animate" />
        </div>
      </div>

      <style>{`
        .progress-bar-animate {
          animation: progress 3s ease-in-out forwards;
        }
        @keyframes progress {
          0% { width: 0%; }
          20% { width: 35%; }
          60% { width: 80%; }
          100% { width: 100%; }
        }
      `}</style>
    </div>
  );

  return createPortal(overlayContent, document.body);
}

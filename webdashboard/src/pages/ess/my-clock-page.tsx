import { useQuery } from "@tanstack/react-query";
import { UserX } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import {
  AdjustmentData,
  AdjustmentRequestModal,
} from "@/components/time-tracking/adjustment-request-modal";
import { PunchCard } from "@/components/time-tracking/punch-card";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useMyClock, useMyProfile } from "@/hooks/use-ess";
import { fetchWorkspace, getCurrentWorkspaceId, submitAttendanceAdjustment } from "@/lib/api";
import { toUTCISO } from "@/lib/datetime";

import { HistoryTable } from "./components/history-table";
import { MyClockSkeleton } from "./components/my-clock-skeleton";

export default function MyClockPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();

  const { employee, isLoading: isLoadingProfile } = useMyProfile();
  const {
    activeEntry,
    entries,
    isLoading: isLoadingClock,
    isPunching,
    punch,
  } = useMyClock();

  const { data: workspace } = useQuery({
    enabled: !!wsId,
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchWorkspace(wsId);
    },
    queryKey: ["workspace", wsId],
  });

  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);

  const handlePunch = async () => {
    try {
      await punch({});
      toast.success(t("ess.clock.punch_success", "Chấm công thành công"));
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : "Unknown error";
      toast.error(message);
    }
  };

   
  const handleAdjustmentSubmit = async (data: AdjustmentData) => {
    if (!wsId || !employee) {
      toast.error(t("common.error"));
      return;
    }

    try {
      const tz = workspace?.timezone ?? "UTC";
      const punchTime = toUTCISO(data.date, data.punchTime, tz);

      await submitAttendanceAdjustment(wsId, {
        employment_id: employee.active_employment_id ?? "",
        reason: data.reason,
        requested_punch_time: punchTime,
        time_entry_id: data.timeEntryId,
      });
      toast.success(t("ess.clock.adjustment_requested"));
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : "Unknown error";
      toast.error(message);
    }
  };

  if (isLoadingProfile || isLoadingClock) return <MyClockSkeleton />;

  if (!employee) return <EmptyState t={t} />;

  return (
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <PageHeader data-testid="ess-clock-title" subtitle={t("ess.clock.subtitle")} title={t("ess.clock.title")} />

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        <div className="lg:col-span-5">
          <PunchCard
            activeEntry={activeEntry}
            isPunching={isPunching}
            onPunch={handlePunch}
            onRequestAdjustment={() => {
              setIsAdjustmentModalOpen(true);
            }}
            userName={employee.full_name}
          />
        </div>

        <HistoryTable entries={entries} />
      </div>

      <AdjustmentRequestModal
        isOpen={isAdjustmentModalOpen}
        onClose={() => {
          setIsAdjustmentModalOpen(false);
        }}
        onSubmit={handleAdjustmentSubmit}
        timezone={workspace?.timezone ?? "UTC"}
      />
    </div>
  );
}

function EmptyState({ t }: { t: TFunction }) {
  return (
    <div className="py-24 text-center border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center max-w-lg mx-auto mt-10">
      <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm dark:shadow-black/10 mb-6 border border-slate-100 dark:border-slate-800 text-slate-350 dark:text-slate-650">
        <UserX size={32} />
      </div>
      <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-[0.25em] mb-2" variant="p">
        {t("common.no_profile")}
      </Typography>
      <Typography className="text-xs text-slate-400 dark:text-slate-500 max-w-xs leading-relaxed px-6" variant="p">
        Please contact your administrator to set up your employee profile.
      </Typography>
    </div>
  );
}



import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import {
  AdjustmentData,
  AdjustmentRequestModal,
} from "@/components/time-tracking/adjustment-request-modal";
import { PunchCard } from "@/components/time-tracking/punch-card";
import { useToast } from "@/components/ui/toast";
import { useMyClock, useMyProfile } from "@/hooks/use-ess";
import { fetchWorkspace, getCurrentWorkspaceId, submitAttendanceAdjustment } from "@/lib/api";
import { toUTCISO } from "@/lib/datetime";

import { HistoryTable } from "./components/history-table";

export default function MyClockPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();

  const { employee, isLoading: isLoadingProfile } = useMyProfile();
  const {
    activeEntry,
    entries,
    isLoading: isLoadingClock,
    isOnBreak,
    isPunching,
    punchIn,
    punchOut,
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

  const handlePunchIn = async (type: "work" | "break") => {
    try {
      await punchIn({ entry_type: type });
      toast.success(
        type === "work" ? t("ess.clock.punch_in_success") : t("ess.clock.break_start_success")
      );
    } catch {
      toast.error(t("common.error"));
    }
  };

  const handlePunchOut = async () => {
    try {
      await punchOut({});
      toast.success(t("ess.clock.punch_out_success"));
    } catch {
      toast.error(t("common.error"));
    }
  };

  const handleAdjustmentSubmit = async (data: AdjustmentData) => {
    if (!wsId || !employee) {
      toast.error(t("common.error"));
      return;
    }

    try {
      const tz = workspace?.timezone ?? "UTC";
      const punchInTime = toUTCISO(data.date, data.punchIn, tz);
      const punchOutTime = toUTCISO(data.date, data.punchOut, tz);

      await submitAttendanceAdjustment(wsId, {
        employee_id: employee.id,
        reason: data.reason,
        requested_punch_in_time: punchInTime,
        requested_punch_out_time: punchOutTime,
        time_entry_id: data.timeEntryId,
      });
      toast.success(t("ess.clock.adjustment_requested"));
    } catch {
      toast.error(t("common.error"));
    }
  };

  if (isLoadingProfile || isLoadingClock) return <LoadingState />;

  if (!employee) return <EmptyState t={t} />;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <Header t={t} />

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        <div className="lg:col-span-5">
          <PunchCard
            activeEntry={activeEntry}
            isOnBreak={isOnBreak}
            isPunching={isPunching}
            onPunchIn={handlePunchIn}
            onPunchOut={handlePunchOut}
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

function LoadingState() {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
    </div>
  );
}

function EmptyState({ t }: { t: any }) {
  return (
    <div className="p-8 text-center text-slate-500 dark:text-slate-400 font-bold italic">
      {t("common.no_profile")}
    </div>
  );
}

function Header({ t }: { t: any }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div>
        <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">
          {t("ess.clock.title")}
        </h1>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60 mt-1.5">
          {t("ess.clock.subtitle")}
        </p>
      </div>
    </div>
  );
}

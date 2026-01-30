import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { History } from "lucide-react";
import { useMyProfile, useMyClock } from "@/hooks/use-ess";
import { useToast } from "@/components/ui/Toast";
import { fetchWorkspace, getCurrentWorkspaceId, submitAttendanceAdjustment } from "@/lib/api";
import { toUTCISO } from "@/lib/datetime";
import { PunchCard } from "@/components/time-tracking/PunchCard";
import { TimeEntryRow } from "@/components/time-tracking/TimeEntryRow";
import {
  AdjustmentRequestModal,
  AdjustmentData,
} from "@/components/time-tracking/AdjustmentRequestModal";

export default function MyClockPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();

  const { employee, isLoading: isLoadingProfile } = useMyProfile();
  const {
    entries,
    activeEntry,
    isOnBreak,
    isLoading: isLoadingClock,
    punchIn,
    punchOut,
    isPunching,
  } = useMyClock();

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);

  const handlePunchIn = async (type: "work" | "break") => {
    try {
      await punchIn({ entry_type: type });
      toast.success(
        type === "work"
          ? t("ess.clock.punch_in_success")
          : t("ess.clock.break_start_success")
      );
    } catch (error) {
      toast.error(t("common.error"));
    }
  };

  const handlePunchOut = async () => {
    try {
      await punchOut({});
      toast.success(t("ess.clock.punch_out_success"));
    } catch (error) {
      toast.error(t("common.error"));
    }
  };

  const handleAdjustmentSubmit = async (data: AdjustmentData) => {
    if (!wsId || !employee) {
      toast.error(t("common.error"));
      return;
    }

    try {
      const tz = workspace?.timezone || "UTC";
      const punchInTime = toUTCISO(data.date, data.punchIn, tz);
      const punchOutTime = toUTCISO(data.date, data.punchOut, tz);

      await submitAttendanceAdjustment(wsId, {
        employee_id: employee.id,
        time_entry_id: data.timeEntryId,
        requested_punch_in_time: punchInTime,
        requested_punch_out_time: punchOutTime,
        reason: data.reason,
      });
      toast.success(t("ess.clock.adjustment_requested"));
    } catch (error) {
      toast.error(t("common.error"));
    }
  };

  if (isLoadingProfile || isLoadingClock) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  if (!employee) {
    return (
      <div className="p-8 text-center text-slate-500 font-bold italic">
        {t("common.no_profile")}
      </div>
    );
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div>
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 uppercase leading-none">
            {t("ess.clock.title")}
          </h1>
          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] opacity-60 mt-1.5">
            {t("ess.clock.subtitle")}
          </p>
        </div>
      </div>

      {/* Main Content: 2 Columns on Desktop, Stack on Mobile */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* Left: Punch Card */}
        <div className="lg:col-span-5">
          <PunchCard
            activeEntry={activeEntry}
            isOnBreak={isOnBreak}
            userName={employee.full_name}
            onPunchIn={handlePunchIn}
            onPunchOut={handlePunchOut}
            onRequestAdjustment={() => setIsAdjustmentModalOpen(true)}
            isPunching={isPunching}
          />
        </div>

        {/* Right: Quick History */}
        <div className="lg:col-span-7 bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col">
          <div className="px-8 py-6 border-b border-slate-50 bg-slate-50/30 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <History className="w-5 h-5 text-primary-500" />
              <h3 className="font-bold text-slate-800 uppercase tracking-widest text-xs">
                {t("ess.clock.recent_history")}
              </h3>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto max-h-[600px]">
            <table className="w-full text-left border-collapse">
              <thead className="sticky top-0 bg-white/80 backdrop-blur-md z-10 border-b border-slate-100">
                <tr>
                  <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("common.date")}
                  </th>
                  <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("common.in")}
                  </th>
                  <th className="px-8 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("common.out")}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {entries.length === 0 ? (
                  <tr>
                    <td
                      colSpan={3}
                      className="px-10 py-20 text-center text-slate-500 font-bold italic uppercase tracking-widest text-xs opacity-50"
                    >
                      {t("ess.clock.no_entries")}
                    </td>
                  </tr>
                ) : (
                  entries.map((entry) => (
                    <TimeEntryRow
                      key={entry.id}
                      entry={entry}
                      showAdjustButton={false}
                    />
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Adjustment Request Modal */}
      <AdjustmentRequestModal
        isOpen={isAdjustmentModalOpen}
        onClose={() => setIsAdjustmentModalOpen(false)}
        onSubmit={handleAdjustmentSubmit}
        timezone={workspace?.timezone || "UTC"}
      />
    </div>
  );
}

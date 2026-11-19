import { Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import type { IntervalDraft } from "../hooks/use-shift-editor";

import IntervalDraftEditor from "./interval-draft-editor";

interface IntervalEditorCardProps {
  intervalDrafts: IntervalDraft[];
  onAddInterval: () => void;
  onRemoveInterval: (key: string) => void;
  onUpdateInterval: (key: string, patch: Partial<IntervalDraft>) => void;
}

export function IntervalEditorCard({
  intervalDrafts,
  onAddInterval,
  onRemoveInterval,
  onUpdateInterval,
}: IntervalEditorCardProps) {
  const { t } = useTranslation();

  return (
    <section className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-10 shadow-sm space-y-6" aria-labelledby="shift-work-periods-title">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-50 dark:border-slate-800/80 pb-4">
        <div>
          <Typography
            className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]"
            id="shift-work-periods-title"
            variant="h3"
          >
            {t("work_segments.intervals.title", "Giờ làm việc")}
          </Typography>
          <p className="mt-2 text-sm text-slate-500 dark:text-slate-400 max-w-xl">
            {t(
              "work_segments.intervals.helper",
              "Một khoảng làm việc là thời gian làm việc liên tục. Ví dụ, 07:30–16:30 là một khoảng, kể cả khi có nghỉ giữa ca.",
            )}
          </p>
        </div>
        <Button
          className="text-xs min-h-11"
          onClick={onAddInterval}
          size="sm"
          type="button"
          variant="outline"
        >
          + {t("work_segments.intervals.add_interval", "Thêm khoảng làm việc")}
        </Button>
      </div>

      {intervalDrafts.length === 0 ? (
        <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-sm italic">
          <Info aria-hidden="true" className="mx-auto mb-2 opacity-40" size={24} />
          {t(
            "work_segments.intervals.no_intervals",
            "Chưa có khoảng làm việc. Hãy thêm ít nhất một khoảng.",
          )}
        </div>
      ) : (
        <div className="space-y-6">
          {intervalDrafts.map((iv, idx) => (
            <IntervalDraftEditor
              index={idx}
              interval={iv}
              key={iv.key}
              onRemove={onRemoveInterval}
              onUpdate={onUpdateInterval}
              showRemove={intervalDrafts.length > 1}
            />
          ))}
        </div>
      )}

      {intervalDrafts.length === 1 ? (
        <p className="text-xs text-slate-500 dark:text-slate-400">
          {t(
            "work_segments.intervals.split_help",
            "Chỉ thêm khoảng thứ hai khi thời gian làm việc bị tách rời, ví dụ 07:30–11:30 và 13:00–17:00.",
          )}
        </p>
      ) : null}
    </section>
  );
}

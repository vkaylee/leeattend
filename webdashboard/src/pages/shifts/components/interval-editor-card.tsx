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
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-10 shadow-sm space-y-6">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800/80 pb-4">
        <Typography
          className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]"
          variant="h3"
        >
          {t("work_segments.intervals.title", "Khung giờ trong ca").toUpperCase()}
        </Typography>
        <Button
          className="text-xs"
          onClick={onAddInterval}
          size="sm"
          type="button"
          variant="outline"
        >
          + {t("work_segments.intervals.add_interval", "Thêm khung giờ")}
        </Button>
      </div>

      {intervalDrafts.length === 0 ? (
        <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-sm italic">
          <Info className="mx-auto mb-2 opacity-40" size={24} />
          {t(
            "work_segments.intervals.no_intervals",
            "Chưa có khung giờ. Hệ thống dùng giờ bắt đầu/kết thúc của ca.",
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
    </div>
  );
}

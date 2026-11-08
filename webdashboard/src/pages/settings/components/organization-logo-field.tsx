import { ImagePlus, Trash2, UploadCloud, Check, X } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface OrganizationLogoFieldProps {
  labelClassName?: string;
  logoUrl: string;
  onLogoChange: (url: string) => void;
}

export function OrganizationLogoField({
  labelClassName = "text-xs font-semibold text-slate-700 dark:text-slate-300 ml-0.5",
  logoUrl,
  onLogoChange,
}: OrganizationLogoFieldProps) {
  const { t } = useTranslation();
  const [isEditing, setIsEditing] = useState(false);
  const [tempUrl, setTempUrl] = useState(logoUrl);

  const handleApply = () => {
    onLogoChange(tempUrl.trim());
    setIsEditing(false);
  };

  const handleCancel = () => {
    setTempUrl(logoUrl);
    setIsEditing(false);
  };

  const handleStartEdit = () => {
    setTempUrl(logoUrl);
    setIsEditing(true);
  };

  return (
    <div className="space-y-2">
      <label className={labelClassName}>
        {t("common.workspace_settings.workspace_logo", {
          defaultValue: "Logo tổ chức",
        })}
      </label>

      {isEditing ? (
        <div className="rounded-xl border border-primary-300 dark:border-primary-700 bg-primary-50/30 dark:bg-primary-950/20 p-3 space-y-2.5">
          <div className="text-[11px] font-medium text-slate-600 dark:text-slate-400">
            {t("common.workspace_settings.enter_logo_url", {
              defaultValue: "Nhập URL hình ảnh logo (HTTPS):",
            })}
          </div>
          <div className="flex items-center gap-2">
            <input
              aria-label={t("common.workspace_settings.workspace_logo", {
                defaultValue: "Logo URL",
              })}
              autoFocus
              className="flex-1 h-9 px-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-lg text-xs font-mono text-slate-800 dark:text-slate-100 outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500"
              onChange={(e) => setTempUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleApply();
                if (e.key === "Escape") handleCancel();
              }}
              placeholder="https://example.com/logo.png"
              type="url"
              value={tempUrl}
            />
            <Button
              className="h-9 px-3 text-xs normal-case tracking-normal"
              icon={<Check size={14} />}
              onClick={handleApply}
              size="sm"
              variant="primary"
            >
              {t("common.apply", { defaultValue: "Áp dụng" })}
            </Button>
            <Button
              className="h-9 px-2.5 text-xs text-slate-500 normal-case tracking-normal"
              icon={<X size={14} />}
              onClick={handleCancel}
              size="sm"
              variant="ghost"
            >
              {t("common.cancel", { defaultValue: "Hủy" })}
            </Button>
          </div>
        </div>
      ) : logoUrl ? (
        <div className="relative group rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50 flex flex-col items-center justify-center p-4 h-32">
          <div
            aria-label={t("common.workspace_settings.workspace_logo", { defaultValue: "Logo tổ chức" })}
            className="h-16 w-full bg-contain bg-no-repeat bg-center drop-shadow-2xs"
            role="img"
            style={{ backgroundImage: `url(${logoUrl})` }}
          />
          <div className="absolute inset-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 rounded-xl">
            <Button
              className="p-2 bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-lg shadow-2xs hover:text-primary-600 border border-slate-200 dark:border-slate-700 transition-colors"
              data-testid="edit-logo-btn"
              onClick={handleStartEdit}
              title={t("common.edit", { defaultValue: "Edit" })}
              variant="ghost"
            >
              <ImagePlus size={16} />
            </Button>
            <Button
              className="p-2 bg-white dark:bg-slate-800 text-rose-500 rounded-lg shadow-2xs hover:bg-rose-50 border border-slate-200 dark:border-slate-700 transition-colors"
              data-testid="delete-logo-btn"
              onClick={() => onLogoChange("")}
              title={t("common.delete", { defaultValue: "Delete" })}
              variant="ghost"
            >
              <Trash2 size={16} />
            </Button>
          </div>
        </div>
      ) : (
        <Button
          className="w-full h-32 rounded-xl border-2 border-dashed border-slate-200 dark:border-slate-700 hover:border-primary-400 dark:hover:border-primary-500 bg-slate-50 dark:bg-slate-800/30 hover:bg-primary-50/50 dark:hover:bg-primary-900/10 transition-all flex flex-col items-center justify-center gap-2 group"
          data-testid="upload-logo-btn"
          onClick={handleStartEdit}
          variant="ghost"
        >
          <div className="w-9 h-9 rounded-full bg-white dark:bg-slate-800 shadow-2xs flex items-center justify-center text-slate-400 group-hover:text-primary-500 transition-all">
            <UploadCloud size={18} />
          </div>
          <div className="text-center">
            <Typography
              className="text-xs font-semibold text-slate-600 dark:text-slate-300 group-hover:text-primary-600"
              variant="p"
            >
              {t("common.workspace_settings.upload_logo", {
                defaultValue: "Nhấp để cài đặt logo URL",
              })}
            </Typography>
          </div>
        </Button>
      )}
    </div>
  );
}

export default OrganizationLogoField;

import { Database, Save, CheckCircle2, XCircle, Link as LinkIcon, Server, Key, Shield } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { SettingsSwitch } from "@/components/ui/settings/settings-switch";
import { Typography } from "@/components/ui/typography";

import type { FormState } from "./storage-types";

interface StorageHeaderProps {
  isSaving: boolean;
  onSave: () => void;
}

export function StorageHeader({ isSaving, onSave }: StorageHeaderProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-2xs border border-primary-100/50 dark:border-primary-900/50">
          <Database size={20} />
        </div>
        <div>
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h3">
            {t("settings.storage.title", "Cấu hình lưu trữ")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
            {t("settings.storage.description", "Cấu hình S3/MinIO lưu trữ tệp tài liệu và hồ sơ")}
          </Typography>
        </div>
      </div>
      <Button
        className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs"
        data-testid="save-storage-btn"
        icon={<Save size={15} />}
        loading={isSaving}
        onClick={onSave}
        variant="primary"
      >
        {t("common.save_changes", "Lưu thay đổi")}
      </Button>
    </div>
  );
}

interface StorageCredentialFormProps {
  form: FormState;
  hasExistingConfig: boolean;
  isSaving: boolean;
  isTesting: boolean;
  onFieldChange: (updater: (prev: FormState) => FormState) => void;
  onTestConnection: () => void;
  onToggleCustom: (enabled: boolean) => void;
  testResult: "success" | "error" | null;
}

export function StorageCredentialForm({
  form,
  hasExistingConfig,
  isTesting,
  isSaving,
  testResult,
  onFieldChange,
  onTestConnection,
  onToggleCustom,
}: StorageCredentialFormProps) {
  const { t } = useTranslation();

  const handleToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    onToggleCustom(e.target.checked);
  };

  return (
    <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
      {/* Toggle Custom Storage */}
      <div className="flex flex-col md:flex-row md:items-center justify-between p-5 bg-slate-50 dark:bg-slate-800/60 rounded-xl border border-slate-200/70 dark:border-slate-700/70 gap-4">
        <div className="space-y-0.5">
          <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="h4">
            {t("settings.storage.use_custom", "Sử dụng Bucket lưu trữ riêng")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
            {t("settings.storage.use_custom_desc", "Lưu trữ tệp tại AWS S3 hoặc MinIO do doanh nghiệp tự quản lý.")}
          </Typography>
        </div>
        <SettingsSwitch
          ariaLabel={t("settings.storage.use_custom", "Use Custom Storage Bucket")}
          checked={form.isCustom}
          id="use-custom-storage"
          onCheckedChange={onToggleCustom}
        />
      </div>

      {/* Custom Storage Fields */}
      {form.isCustom ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 animate-in slide-in-from-top-4 duration-300">
          <div className="col-span-full border-b border-slate-50 dark:border-slate-800 pb-4">
            <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">
              {t("settings.storage.s3_credentials", "S3 Credentials")}
            </Typography>
          </div>

          <div className="space-y-3 col-span-full">
            <label className={LABEL_BASE}>{t("settings.storage.endpoint", "Endpoint URL")}</label>
            <div className="relative">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><LinkIcon size={18} /></div>
              <input
                className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, endpointUrl: e.target.value })); }}
                placeholder="https://s3.amazonaws.com or https://play.min.io:9000"
                type="url"
                value={form.endpointUrl}
              />
            </div>
          </div>

          <div className="space-y-3">
            <label className={LABEL_BASE}>{t("settings.storage.bucket", "Bucket Name")}</label>
            <div className="relative">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Database size={18} /></div>
              <input
                className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, bucketName: e.target.value })); }}
                placeholder="my-company-assets"
                type="text"
                value={form.bucketName}
              />
            </div>
          </div>

          <div className="space-y-3">
            <label className={LABEL_BASE}>{t("settings.storage.region", "Region")}</label>
            <div className="relative">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Server size={18} /></div>
              <input
                className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, region: e.target.value })); }}
                placeholder="us-east-1"
                type="text"
                value={form.region}
              />
            </div>
          </div>

          <div className="space-y-3">
            <label className={LABEL_BASE}>{t("settings.storage.access_key", "Access Key")}</label>
            <div className="relative">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Key size={18} /></div>
              <input
                className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, accessKey: e.target.value })); }}
                placeholder="AKIAIOSFODNN7EXAMPLE"
                type="text"
                value={form.accessKey}
              />
            </div>
          </div>

          <div className="space-y-3">
            <label className={LABEL_BASE}>{t("settings.storage.secret_key", "Secret Key")}</label>
            <div className="relative">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Shield size={18} /></div>
              <input
                className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, secretKey: e.target.value })); }}
                placeholder={hasExistingConfig ? "•••••••••••••••• (Leave blank to keep existing)" : "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"}
                type="password"
                value={form.secretKey}
              />
            </div>
          </div>

          <div className="col-span-full pt-2">
            <label className="flex items-center gap-3 cursor-pointer p-4 bg-slate-50 dark:bg-slate-800/50 rounded-[1.5rem] border border-slate-100 dark:border-slate-700 hover:border-primary-200 transition-colors" htmlFor="use-path-style">
              <span className="sr-only">{t("settings.storage.path_style", "Use Path-Style URLs")}</span>
              <input
                checked={form.usePathStyle}
                className="w-5 h-5 text-primary-600 rounded border-slate-300 focus:ring-primary-500"
                id="use-path-style"
                onChange={(e) => { onFieldChange(prev => ({ ...prev, usePathStyle: e.target.checked })); }}
                type="checkbox"
              />
              <div>
                <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200">{t("settings.storage.path_style", "Use Path-Style URLs")}</Typography>
                <Typography className="text-[10px] text-slate-500 dark:text-slate-400" variant="p">{t("settings.storage.path_style_desc", "Enable this if you are using MinIO or a local S3 clone. Disable for AWS S3 (Virtual Hosted-Style).")}</Typography>
              </div>
            </label>
          </div>

          <TestConnectionSection isSaving={isSaving} isTesting={isTesting} onTestConnection={onTestConnection} t={t} testResult={testResult} />
        </div>
      ) : null}
    </div>
  );
}

function TestConnectionSection({
  isTesting,
  isSaving,
  testResult,
  onTestConnection,
  t,
}: {
  isTesting: boolean;
  isSaving: boolean;
  testResult: "success" | "error" | null;
  onTestConnection: () => void;
  t: (key: string) => string;
}) {
  return (
    <div className="col-span-full flex items-center gap-4 mt-6 p-6 bg-slate-50 dark:bg-slate-800/30 rounded-[1.5rem] border border-slate-100 dark:border-slate-800">
      <Button
        disabled={isTesting || isSaving}
        loading={isTesting}
        onClick={onTestConnection}
        variant="outline"
      >
        {t("settings.storage.test_btn").toUpperCase()}
      </Button>
      {testResult === "success" ? (
        <span className="flex items-center text-sm font-bold text-emerald-600 dark:text-emerald-400 gap-2 animate-in fade-in bg-emerald-50 dark:bg-emerald-950/30 px-4 py-2 rounded-xl">
          <CheckCircle2 className="w-5 h-5" />
          {t("settings.storage.test_success")}
        </span>
      ) : null}
      {testResult === "error" ? (
        <span className="flex items-center text-sm font-bold text-rose-600 dark:text-rose-400 gap-2 animate-in fade-in bg-rose-50 dark:bg-rose-950/30 px-4 py-2 rounded-xl">
          <XCircle className="w-5 h-5" />
          {t("settings.storage.test_error")}
        </span>
      ) : null}
    </div>
  );
}

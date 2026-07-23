import { Mail, Save, ShieldCheck, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { SmtpConfigData } from "@/lib/api/smtp-config";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import PageHeader from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";

import { SmtpHttpApiSection } from "./components/smtp-http-api-section";
import { SmtpTestCard } from "./components/smtp-test-card";
import { useSmtpConfig } from "./hooks/use-smtp-config";

export default function SmtpSettingsPage() {
  const { t } = useTranslation();
  const {
    isLoading,
    dirty,
    testEmail,
    setTestEmail,
    showHttpApi,
    setShowHttpApi,
    showDefaultConfirm,
    setShowDefaultConfirm,
    testResult,
    setTestResult,
    testing,
    verifiedAt,
    verifiedTime,
    form,
    httpForm,
    setHttpOverrides,
    handleFillDefault,
    updateField,
    saveMutation,
    handleSave,
    handleTest,
    markDirty,
  } = useSmtpConfig();

  const fields: {
    key: keyof SmtpConfigData;
    label: string;
    type: string;
    placeholder: string;
    hint?: string;
  }[] = [
    { key: "host", label: t("smtp.host", "SMTP Host"), type: "text", placeholder: "smtp.gmail.com" },
    { key: "port", label: t("smtp.port", "Port"), type: "number", placeholder: "587", hint: "465 = TLS, 587 = STARTTLS" },
    { key: "username", label: t("smtp.username", "Username"), type: "text", placeholder: "admin@company.com" },
    { key: "password", label: t("smtp.password", "Password / App Password"), type: "password", placeholder: "••••••••" },
    { key: "from", label: t("smtp.from", "From Address"), type: "text", placeholder: "LeeAttend <noreply@leeattend.com>" },
    { key: "base_url", label: t("smtp.base_url", "App Base URL"), type: "text", placeholder: "https://app.leeattend.com", hint: t("smtp.base_url_hint", "Used in password reset links") },
  ];

  if (isLoading) {
    return (
      <div className="space-y-5 p-8">
        <Skeleton className="h-8 w-64" />
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton className="h-12 w-full" key={i} />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-6 p-8 animate-in fade-in duration-300">
      {/* Header with verification badge */}
      <div className="space-y-1">
        <PageHeader
          icon={<Mail className="w-5 h-5" />}
          subtitle={t("smtp.description", "Configure outgoing email for password resets and notifications.")}
          title={t("smtp.title", "SMTP Email Settings")}
        />
        {verifiedAt ? (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
            <ShieldCheck className="w-3.5 h-3.5" />
            {t("smtp.verified_badge", "Verified")} — {verifiedTime}
          </span>
        ) : (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-800">
            <AlertTriangle className="w-3.5 h-3.5" />
            {t("smtp.untested_badge", "Untested — send a test email to verify")}
          </span>
        )}
      </div>

      {/* Config form card */}
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl p-6 space-y-5 shadow-sm">
        <Typography className="font-semibold text-slate-700 dark:text-slate-200" variant="h3">
          {t("smtp.config_section", "SMTP Configuration")}
        </Typography>

        <div className="grid gap-4 sm:grid-cols-2">
          {fields.map(({ key, label, type, placeholder, hint }) => (
            <div className="space-y-1.5" key={key}>
              <label className="text-sm font-medium text-slate-600 dark:text-slate-400">{label}</label>
              <input
                className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                data-testid={`smtp-field-${key}`}
                onChange={(e) => {
                  const val = type === "number" ? Number(e.target.value) || 0 : e.target.value;
                  updateField(key, val);
                }}
                placeholder={placeholder}
                type={type}
                value={String(form[key] ?? "")}
              />
              {hint ? (
                <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">{hint}</Typography>
              ) : null}
            </div>
          ))}
        </div>

        <SmtpHttpApiSection
          expanded={showHttpApi}
          form={httpForm}
          onFieldChange={(field, value) => {
            setHttpOverrides((p) => ({ ...p, [field]: value }));
            markDirty();
          }}
          onFillDefault={handleFillDefault}
          onToggle={() => { setShowHttpApi((p) => !p); }}
        />

        <div className="flex justify-end pt-2">
          <Button
            data-testid="smtp-save-btn"
            disabled={!dirty || saveMutation.isPending}
            loading={saveMutation.isPending}
            onClick={handleSave}
            variant="secondary"
          >
            <Save className="w-4 h-4 mr-1.5" />
            {t("common.save")}
          </Button>
        </div>
      </div>

      <SmtpTestCard
        onTest={handleTest}
        onTestEmailChange={(email) => {
          setTestEmail(email);
          setTestResult(null);
        }}
        testEmail={testEmail}
        testing={testing}
        testResult={testResult}
        verifiedAt={verifiedAt}
      />

      <ConfirmDialog
        confirmLabel={t("smtp.fill_default_confirm", "Apply Default")}
        isOpen={showDefaultConfirm}
        message={t(
          "smtp.fill_default_warning",
          "This will reset ALL HTTP API fields to standard default values (Mailtrap-compatible). Any custom configuration will be lost. Continue?",
        )}
        onClose={() => { setShowDefaultConfirm(false); }}
        onConfirm={handleFillDefault}
        title={t("smtp.fill_default_title", "Reset to Default Template?")}
        variant="warning"
      />
    </div>
  );
}

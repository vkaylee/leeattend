import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Mail, Save, Send, CheckCircle, XCircle, Loader2, ShieldCheck, AlertTriangle } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  fetchSmtpConfig,
  saveSmtpConfig,
  testSmtp,
  type SmtpConfigData,
} from "@/lib/api/smtp-config";

const EMPTY_CONFIG: SmtpConfigData = {
  host: "",
  port: 587,
  username: "",
  password: "",
  from: "",
  base_url: "",
};

export default function SmtpSettingsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: config, isLoading } = useQuery({
    queryKey: ["smtp-config"],
    queryFn: fetchSmtpConfig,
  });

  const [form, setForm] = useState<SmtpConfigData>(EMPTY_CONFIG);
  const [dirty, setDirty] = useState(false);
  const [testEmail, setTestEmail] = useState("");
  const [testResult, setTestResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);
  const [testing, setTesting] = useState(false);
  const [initialized, setInitialized] = useState(false);
  // Track verification status: null = untested, Date = last successful test
  const [verifiedAt, setVerifiedAt] = useState<Date | null>(null);

  // Sync form when data loads (one-shot)
  useEffect(() => {
    if (!initialized && config) {
      setForm({
        host: config.host ?? "",
        port: config.port ?? 587,
        username: config.username ?? "",
        password: config.password ?? "",
        from: config.from ?? "",
        base_url: config.base_url ?? "",
      });
      setInitialized(true);
    }
  }, [config, initialized]);

  const updateField = (field: keyof SmtpConfigData, value: string | number) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setDirty(true);
  };

  const saveMutation = useMutation({
    mutationFn: saveSmtpConfig,
    onSuccess: () => {
      toast.success(t("common.saved_success"));
      void queryClient.invalidateQueries({ queryKey: ["smtp-config"] });
      setDirty(false);
      // Config changed → verification is stale
      setVerifiedAt(null);
      setTestResult(null);
    },
    onError: (err: Error) => {
      toast.error(err.message);
    },
  });

  const handleSave = () => {
    saveMutation.mutate({
      host: form.host || undefined,
      port: form.port || undefined,
      username: form.username || undefined,
      password: form.password || undefined,
      from: form.from || undefined,
      base_url: form.base_url || undefined,
    });
  };

  const handleTest = async () => {
    if (!testEmail.trim()) return;
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testSmtp(testEmail.trim());
      setTestResult(result);
      if (result.success) {
        setVerifiedAt(new Date());
      }
    } catch (err: unknown) {
      setTestResult({
        success: false,
        message: err instanceof Error ? err.message : "Test failed",
      });
    } finally {
      setTesting(false);
    }
  };

  const fields: Array<{
    key: keyof SmtpConfigData;
    label: string;
    type: string;
    placeholder: string;
    hint?: string;
  }> = [
    {
      key: "host",
      label: t("smtp.host", "SMTP Host"),
      type: "text",
      placeholder: "smtp.gmail.com",
    },
    {
      key: "port",
      label: t("smtp.port", "Port"),
      type: "number",
      placeholder: "587",
      hint: "465 = TLS, 587 = STARTTLS",
    },
    {
      key: "username",
      label: t("smtp.username", "Username"),
      type: "text",
      placeholder: "admin@company.com",
    },
    {
      key: "password",
      label: t("smtp.password", "Password / App Password"),
      type: "password",
      placeholder: "••••••••",
    },
    {
      key: "from",
      label: t("smtp.from", "From Address"),
      type: "text",
      placeholder: "LeeAttend <noreply@leeattend.com>",
    },
    {
      key: "base_url",
      label: t("smtp.base_url", "App Base URL"),
      type: "text",
      placeholder: "https://app.leeattend.com",
      hint: t("smtp.base_url_hint", "Used in password reset links"),
    },
  ];

  const verifiedTime = useMemo(() => {
    if (!verifiedAt) return null;
    return verifiedAt.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  }, [verifiedAt]);

  if (isLoading) {
    return (
      <div className="space-y-5 p-8">
        <Skeleton className="h-8 w-64" />
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton key={i} className="h-12 w-full" />
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
              <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                {label}
              </label>
              <input
                className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                data-testid={`smtp-field-${key}`}
                onChange={(e) => {
                  const val = type === "number" ? Number(e.target.value) || 0 : e.target.value;
                  updateField(key, val);
                }}
                placeholder={placeholder}
                type={type}
                value={type === "number" ? (form[key] as number) || "" : (form[key] as string) ?? ""}
              />
              {hint ? (
                <p className="text-[11px] text-slate-400 dark:text-slate-500">{hint}</p>
              ) : null}
            </div>
          ))}
        </div>

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

      {/* Test card */}
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl p-6 space-y-4 shadow-sm">
        <Typography className="font-semibold text-slate-700 dark:text-slate-200" variant="h3">
          {t("smtp.test_section", "Test Email Delivery")}
        </Typography>
        {!verifiedAt ? (
          <div className="flex items-start gap-2 p-3 rounded-xl bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 text-amber-700 dark:text-amber-300 text-sm">
            <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
            <span>{t("smtp.untested_reminder", "You haven't tested this configuration yet. We strongly recommend sending a test email before relying on this for password resets.")}</span>
          </div>
        ) : null}
        <p className="text-sm text-slate-500 dark:text-slate-400">
          {t("smtp.test_desc", "Send a test email to verify your SMTP configuration works.")}
        </p>

        <div className="flex gap-3 items-end">
          <div className="flex-1 space-y-1.5">
            <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
              {t("smtp.test_email_label", "Send test to")}
            </label>
            <input
              className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
              data-testid="smtp-test-email-input"
              onChange={(e) => {
                setTestEmail(e.target.value);
                setTestResult(null);
              }}
              placeholder="admin@company.com"
              type="email"
              value={testEmail}
            />
          </div>
          <Button
            data-testid="smtp-test-btn"
            disabled={!testEmail.trim() || testing}
            loading={testing}
            onClick={handleTest}
            variant="outline"
          >
            <Send className="w-4 h-4 mr-1.5" />
            {t("smtp.test_btn", "Test Now")}
          </Button>
        </div>

        {testing ? (
          <div className="flex items-center gap-2 text-sm text-slate-500">
            <Loader2 className="w-4 h-4 animate-spin" />
            {t("smtp.testing", "Sending test email...")}
          </div>
        ) : testResult ? (
          <div
            className={`flex items-start gap-2 p-4 rounded-xl text-sm font-medium animate-in fade-in slide-in-from-top-2 duration-200 ${
              testResult.success
                ? "bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300"
                : "bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300"
            }`}
            data-testid="smtp-test-result"
          >
            {testResult.success ? (
              <CheckCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
            ) : (
              <XCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
            )}
            <span className="whitespace-pre-wrap">{testResult.message}</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}

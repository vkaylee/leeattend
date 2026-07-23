import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Mail, Save, Send, CheckCircle, XCircle, Loader2, ShieldCheck, AlertTriangle, ChevronDown, Globe, Sparkles } from "lucide-react";
import { useMemo, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import PageHeader from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  fetchSmtpConfig,
  saveSmtpConfig,
  testSmtp,
  type HttpApiConfig,
  type SmtpConfigData,
} from "@/lib/api/smtp-config";

  // eslint-disable-next-line max-lines-per-function, complexity -- SMTP settings page: form, test dialog, config, logs; genuine layout complexity
export default function SmtpSettingsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: config, isLoading } = useQuery({
    queryKey: ["smtp-config"],
    queryFn: fetchSmtpConfig,
  });

  const [dirty, setDirty] = useState(false);
  const [testEmail, setTestEmail] = useState("");
  // Initialize from config if it already has http_api; UI auto-expands.
  const [showHttpApi, setShowHttpApi] = useState(() => !!config?.http_api);
  const [showDefaultConfirm, setShowDefaultConfirm] = useState(false);

  const DEFAULT_HTTP_API_TEMPLATE = JSON.stringify(
    {
      from: { email: "{{from_email}}", name: "{{from_name}}" },
      to: [{ email: "{{to}}" }],
      subject: "{{subject}}",
      text: "{{body}}",
    },
    null,
    2,
  );
  const [testResult, setTestResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);
  const [testing, setTesting] = useState(false);
  // Track verification status: null = untested, Date = last successful test
  // Derive initial verifiedAt from config; subsequent updates happen in handlers.
  const [verifiedAt, setVerifiedAt] = useState<Date | null>(() =>
    config?.last_verified_at ? new Date(config.last_verified_at) : null,
  );

  // Derive form state from config; only track overrides for dirty tracking
  const [overrides, setOverrides] = useState<Partial<SmtpConfigData>>({});
  const [httpOverrides, setHttpOverrides] = useState<Partial<HttpApiConfig>>({});

  // ── Derived form state from config + overrides ──

  /**
   * Merged SMTP config form. Avoids per-field coalescing boilerplate by
   * spreading the priority chain: overrides → config → defaults.
   */
  function buildForm(cfg: SmtpConfigData | null | undefined, ov: Partial<SmtpConfigData>): SmtpConfigData {
    return {
      host: ov.host ?? cfg?.host ?? "",
      port: ov.port ?? cfg?.port ?? 587,
      username: ov.username ?? cfg?.username ?? "",
      password: ov.password ?? cfg?.password ?? "",
      from: ov.from ?? cfg?.from ?? "",
      base_url: ov.base_url ?? cfg?.base_url ?? "",
      http_api: cfg?.http_api ?? undefined,
    };
  }

  const form: SmtpConfigData = useMemo(() => buildForm(config, overrides), [config, overrides]);

  /** Merged HTTP API config form. Priority: httpOverrides → config.http_api → defaults. */
  function buildHttpForm(
    cfg: SmtpConfigData | null | undefined,
    ov: Partial<SmtpConfigData>,
    hov: Partial<HttpApiConfig>,
  ): HttpApiConfig {
    const src = ov.http_api ?? cfg?.http_api;
    return {
      url: hov.url ?? src?.url ?? "",
      auth_header: hov.auth_header ?? src?.auth_header ?? "",
      auth_value: hov.auth_value ?? src?.auth_value ?? "",
      success_field: hov.success_field ?? src?.success_field ?? "",
      success_value: hov.success_value ?? src?.success_value ?? true,
      body_template: hov.body_template ?? src?.body_template ?? "",
    };
  }

  const httpForm: HttpApiConfig = useMemo(
    () => buildHttpForm(config, overrides, httpOverrides),
    [config, overrides, httpOverrides],
  );

  const handleFillDefault = useCallback(() => {
    setShowDefaultConfirm(false);
    setHttpOverrides({
      url: "https://send.api.mailtrap.io/api/send",
      auth_header: "Api-Token",
      auth_value: "{{password}}",
      success_field: "success",
      success_value: true,
      body_template: DEFAULT_HTTP_API_TEMPLATE,
    });
    setDirty(true);
  }, [DEFAULT_HTTP_API_TEMPLATE]);

  // ── Handlers ──

  const updateField = (field: keyof SmtpConfigData, value: string | number) => {
    setOverrides((prev) => ({ ...prev, [field]: value }));
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

  /** Build save payload from current form state. Pure function — no side effects. */
  function buildSavePayload(smtp: SmtpConfigData, http: HttpApiConfig): SmtpConfigData {
    const payload: SmtpConfigData = {
      host: smtp.host || undefined,
      port: smtp.port || undefined,
      username: smtp.username || undefined,
      password: smtp.password || undefined,
      from: smtp.from || undefined,
      base_url: smtp.base_url || undefined,
    };
    // Include http_api if URL is filled
    if (http.url.trim()) {
      payload.http_api = {
        url: http.url.trim(),
        auth_header: http.auth_header || undefined,
        auth_value: http.auth_value || undefined,
        success_field: http.success_field || undefined,
        success_value: http.success_field ? (http.success_value ?? true) : undefined,
        body_template: http.body_template.trim(),
      };
    } else {
      payload.http_api = null;
    }
    return payload;
  }

  // eslint-disable-next-line complexity -- payload builder with 10+ field coalescence; pure data mapping
  const handleSave = () => {
    const payload = buildSavePayload(form, httpForm);
    saveMutation.mutate(payload);
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

  const fields: {
    key: keyof SmtpConfigData;
    label: string;
    type: string;
    placeholder: string;
    hint?: string;
  }[] = [
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
                value={String(form[key] ?? "")}
              />
              {hint ? (
                <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">{hint}</Typography>
              ) : null}
            </div>
          ))}
        </div>

        {/* HTTP API section */}
        <div className="border-t border-slate-100 dark:border-slate-800 pt-5 mt-2">
          <Button
            className="flex items-center gap-2 w-full !text-left font-semibold !h-auto !px-0 !py-0 !rounded-none !tracking-normal !normal-case"
            data-testid="smtp-http-api-toggle"
            onClick={() => { setShowHttpApi((p) => !p); }}
            variant="ghost"
          >
            <Globe className="w-4 h-4" />
            <Typography variant="h3">
              {t("smtp.http_api_section", "HTTP API (Alternative)")}
            </Typography>
            <ChevronDown
              className={`w-4 h-4 ml-auto transition-transform duration-200 ${
                showHttpApi ? "rotate-180" : ""
              }`}
            />
          </Button>
          <Typography className="text-sm text-slate-400 dark:text-slate-500 mt-1 ml-6" variant="p">
            {t("smtp.http_api_desc", "Use a REST API to send emails instead of SMTP. Required on platforms that block outbound SMTP (Railway, Fly.io, etc.). Supports any transactional email provider.")}
          </Typography>

          <div className="flex justify-end mt-2">
            <Button
              onClick={() => { setShowDefaultConfirm(true); }}
              size="sm"
              variant="ghost"
            >
              <Sparkles className="w-3.5 h-3.5 mr-1" />
              {t("smtp.fill_default", "Fill Default Template")}
            </Button>
          </div>

          {showHttpApi ? (
            <div className="ml-6 mt-4 grid gap-4 sm:grid-cols-2 animate-in fade-in slide-in-from-top-2 duration-200">
              <div className="space-y-1.5 sm:col-span-2">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_url", "API URL")} *
                </label>
                <input
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                  data-testid="smtp-http-api-url"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, url: e.target.value }));
                    setDirty(true);
                  }}
                  placeholder="https://send.api.mailtrap.io/api/send"
                  type="text"
                  value={httpForm.url}
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_auth_header", "Auth Header")}
                </label>
                <input
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                  data-testid="smtp-http-api-auth-header"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, auth_header: e.target.value }));
                    setDirty(true);
                  }}
                  placeholder="Authorization"
                  type="text"
                  value={httpForm.auth_header}
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_auth_value", "Auth Value")}
                </label>
                <input
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                  data-testid="smtp-http-api-auth-value"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, auth_value: e.target.value }));
                    setDirty(true);
                  }}
                  placeholder={'Bearer {{password}}'}
                  type="text"
                  value={httpForm.auth_value}
                />
                <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">
                  {"{{username}} and {{password}} are replaced with SMTP credentials"}
                </Typography>
              </div>

              <div className="space-y-1.5">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_success_field", "Success Field")}
                </label>
                <input
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                  data-testid="smtp-http-api-success-field"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, success_field: e.target.value }));
                    setDirty(true);
                  }}
                  placeholder="success"
                  type="text"
                  value={httpForm.success_field}
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_success_value", "Success Value")}
                </label>
                <input
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
                  data-testid="smtp-http-api-success-value"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, success_value: e.target.value === "false" ? false : e.target.value === "true" ? true : e.target.value }));
                    setDirty(true);
                  }}
                  placeholder="true"
                  type="text"
                  value={typeof httpForm.success_value === "boolean" ? String(httpForm.success_value) : typeof httpForm.success_value === "string" ? httpForm.success_value : ""}
                />
              </div>

              <div className="space-y-1.5 sm:col-span-2">
                <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
                  {t("smtp.http_api_body_template", "JSON Body Template")} *
                </label>
                <textarea
                  className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-slate-600 resize-y"
                  data-testid="smtp-http-api-body-template"
                  onChange={(e) => {
                    setHttpOverrides((p) => ({ ...p, body_template: e.target.value }));
                    setDirty(true);
                  }}
                  placeholder={'{"from":{"email":"{{from_email}}","name":"{{from_name}}"},"to":[{"email":"{{to}}"}],"subject":"{{subject}}","text":"{{body}}"}'}
                  rows={6}
                  value={httpForm.body_template}
                />
                <Typography className="text-[11px] text-slate-400 dark:text-slate-500" variant="p">
                  {"Variables: {{from_name}}, {{from_email}}, {{to}}, {{subject}}, {{body}}"}
                </Typography>
              </div>
            </div>
          ) : null}
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
        <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
          {t("smtp.test_desc", "Send a test email to verify your SMTP configuration works.")}
        </Typography>

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

      {/* Default template confirm dialog */}
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

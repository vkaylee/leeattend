import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import {
  fetchSmtpConfig,
  saveSmtpConfig,
  testSmtp,
  type HttpApiConfig,
  type SmtpConfigData,
} from "@/lib/api/smtp-config";
import { settingsKeys } from "@/lib/query-keys";

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

interface TestResult {
  message: string;
  success: boolean;
}

// ── Pure field-merge helpers (extracted to reduce cyclomatic complexity) ──

/** Merge override > config > default in a single branch point. */
function mergeField<T>(overrideVal: T | undefined, configVal: T | undefined, defaultVal: T): T {
  return overrideVal ?? configVal ?? defaultVal;
}

/** Convert empty string to undefined for API payloads (|| semantics). */
function emptyToUndefined(v: string | undefined): string | undefined {
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- empty string "" must become undefined, ?? won't catch that
  return v || undefined;
}

function buildForm(cfg: SmtpConfigData | null | undefined, ov: Partial<SmtpConfigData>): SmtpConfigData {
  return {
    host: mergeField(ov.host, cfg?.host, ""),
    port: mergeField(ov.port, cfg?.port, 587),
    username: mergeField(ov.username, cfg?.username, ""),
    password: mergeField(ov.password, cfg?.password, ""),
    from: mergeField(ov.from, cfg?.from, ""),
    base_url: mergeField(ov.base_url, cfg?.base_url, ""),
    http_api: cfg?.http_api ?? undefined,
  };
}

function buildHttpForm(
  cfg: SmtpConfigData | null | undefined,
  ov: Partial<SmtpConfigData>,
  hov: Partial<HttpApiConfig>,
): HttpApiConfig {
  const src = ov.http_api ?? cfg?.http_api;
  return {
    url: mergeField(hov.url, src?.url, ""),
    auth_header: mergeField(hov.auth_header, src?.auth_header, ""),
    auth_value: mergeField(hov.auth_value, src?.auth_value, ""),
    success_field: mergeField(hov.success_field, src?.success_field, ""),
    success_value: mergeField(hov.success_value, src?.success_value, true),
    body_template: mergeField(hov.body_template, src?.body_template, ""),
  };
}

function buildSavePayload(smtp: SmtpConfigData, http: HttpApiConfig): SmtpConfigData {
  const payload: SmtpConfigData = {
    host: emptyToUndefined(smtp.host),
    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- port=0 is invalid, must convert to undefined
    port: smtp.port || undefined,
    username: emptyToUndefined(smtp.username),
    password: emptyToUndefined(smtp.password),
    from: emptyToUndefined(smtp.from),
    base_url: emptyToUndefined(smtp.base_url),
  };
  if (http.url.trim()) {
    payload.http_api = {
      url: http.url.trim(),
      auth_header: emptyToUndefined(http.auth_header),
      auth_value: emptyToUndefined(http.auth_value),
      success_field: emptyToUndefined(http.success_field),
      success_value: http.success_field ? (http.success_value ?? true) : undefined,
      body_template: http.body_template.trim(),
    };
  } else {
    payload.http_api = null;
  }
  return payload;
}

export function useSmtpConfig() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatTime } = useDateTimeFormatter();

  const { data: config, isLoading } = useQuery({
    queryKey: settingsKeys.smtp(),
    queryFn: fetchSmtpConfig,
  });

  const [dirty, setDirty] = useState(false);
  const [testEmail, setTestEmail] = useState("");
  const [showHttpApi, setShowHttpApi] = useState(() => !!config?.http_api);
  const [showDefaultConfirm, setShowDefaultConfirm] = useState(false);

  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [testing, setTesting] = useState(false);

  // Derive verifiedAt from two sources:
  //  1. config.last_verified_at (reactive — recomputes when config loads)
  //  2. a post-test-success timestamp (state — set imperatively below)
  // Splitting avoids a setState-in-effect by keeping the config-derived
  // branch purely memoized.
  const configVerifiedAt = useMemo<Date | null>(
    () => (config?.last_verified_at ? new Date(config.last_verified_at) : null),
    [config],
  );
  const [testVerifiedAt, setTestVerifiedAt] = useState<Date | null>(null);
  const verifiedAt = testVerifiedAt ?? configVerifiedAt;

  const [overrides, setOverrides] = useState<Partial<SmtpConfigData>>({});
  const [httpOverrides, setHttpOverrides] = useState<Partial<HttpApiConfig>>({});

  const form: SmtpConfigData = useMemo(() => buildForm(config, overrides), [config, overrides]);
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
  }, []);

  const updateField = (field: keyof SmtpConfigData, value: string | number) => {
    setOverrides((prev) => ({ ...prev, [field]: value }));
    setDirty(true);
  };

  const saveMutation = useMutation({
    mutationFn: saveSmtpConfig,
    onSuccess: () => {
      toast.success(t("common.saved_success"));
      void queryClient.invalidateQueries({ queryKey: settingsKeys.smtp() });
      setDirty(false);
      setTestVerifiedAt(null);
      setTestResult(null);
    },
    onError: (err: Error) => {
      toast.error(err.message);
    },
  });

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
        setTestVerifiedAt(new Date());
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

  const verifiedTime = useMemo(() => {
    if (!verifiedAt) return null;
    return formatTime(verifiedAt);
  }, [verifiedAt, formatTime]);

  return {
    config,
    isLoading,
    dirty,
    markDirty: () => { setDirty(true); },
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
    httpOverrides,
    setHttpOverrides,
    handleFillDefault,
    updateField,
    saveMutation,
    handleSave,
    handleTest,
    t,
  };
}

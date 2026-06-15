/* eslint-disable */
import { Database, Save, CheckCircle2, XCircle, Link as LinkIcon, Server, Key, Shield } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { toast } from "sonner";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import {
  useDeleteStorageConfig,
  useStorageConfig,
  useTestStorageConfig,
  useUpsertStorageConfig,
} from "@/hooks/use-storage-settings";

interface StorageConfig {
  access_key?: string;
  bucket_name?: string;
  endpoint_url?: string;
  region?: string;
  use_path_style?: boolean;
}

export function StorageSettings() {
  const { t } = useTranslation();
  const { data: config, isLoading } = useStorageConfig();
  const upsertMutation = useUpsertStorageConfig();
  const testMutation = useTestStorageConfig();
  const deleteMutation = useDeleteStorageConfig();

  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(null);

  const [isCustom, setIsCustom] = useState(false);
  const [endpointUrl, setEndpointUrl] = useState("");
  const [region, setRegion] = useState("");
  const [bucketName, setBucketName] = useState("");
  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [usePathStyle, setUsePathStyle] = useState(true);

  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    if (config) {
      const c = config as StorageConfig;
      setIsCustom(true);
      setEndpointUrl(c.endpoint_url ?? "");
      setRegion(c.region ?? "");
      setBucketName(c.bucket_name ?? "");
      setAccessKey(c.access_key ?? "");
      setSecretKey(""); // Blank for masked secret
      setUsePathStyle(c.use_path_style ?? true);
    } else {
      setIsCustom(false);
      setEndpointUrl("");
      setRegion("");
      setBucketName("");
      setAccessKey("");
      setSecretKey("");
      setUsePathStyle(true);
    }
  }, [config]);
  /* eslint-enable react-hooks/set-state-in-effect */

  const handleTestConnection = async () => {
    if (!endpointUrl || !bucketName || !accessKey || !secretKey || !region) {
      toast.error(t("settings.storage.test_missing_fields", "Please fill all fields before testing."));
      return;
    }

    setIsTesting(true);
    setTestResult(null);
    try {
      await testMutation.mutateAsync({
        endpoint_url: endpointUrl,
        region,
        bucket_name: bucketName,
        access_key: accessKey,
        secret_key: secretKey,
        use_path_style: usePathStyle,
      });
      setTestResult("success");
      toast.success(t("settings.storage.test_success", "Connection successful!"));
    } catch {
      setTestResult("error");
      toast.error(t("settings.storage.test_error", "Connection failed. Please check your credentials."));
    } finally {
      setIsTesting(false);
    }
  };

  const handleSaveConfig = async () => {
    if (isCustom) {
      try {
        await upsertMutation.mutateAsync({
          endpoint_url: endpointUrl,
          region,
          bucket_name: bucketName,
          access_key: accessKey,
          secret_key: secretKey || undefined, // Send undefined if empty
          use_path_style: usePathStyle,
        });
        toast.success(t("settings.storage.save_success", "Storage configuration saved."));
      } catch {
        toast.error(t("settings.storage.save_error", "Failed to save configuration."));
      }
    } else {
      if (config) {
        try {
          await deleteMutation.mutateAsync();
          toast.success(t("settings.storage.revert_success", "Reverted to Central Storage."));
        } catch {
          toast.error(t("settings.storage.revert_error", "Failed to revert configuration."));
        }
      }
    }
  };

  if (isLoading) {
    return <div className="p-8">Loading...</div>;
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500 max-w-4xl">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
            <Database size={24} />
          </div>
          <div>
            <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" variant="h1">
              {t("settings.storage.title", "Storage Configuration")}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">
              {t("settings.storage.description", "Configure S3-compatible backend. Use Central Storage or your own bucket.")}
            </Typography>
          </div>
        </div>
        <Button 
          data-testid="save-storage-btn" 
          icon={<Save size={18} />} 
          loading={upsertMutation.isPending || deleteMutation.isPending} 
          onClick={handleSaveConfig} 
          variant="primary"
        >
          {t("common.save_changes", "SAVE CHANGES").toUpperCase()}
        </Button>
      </div>

      <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10">
        
        {/* Toggle Custom Storage */}
        <div className="flex flex-col md:flex-row md:items-center justify-between p-6 bg-slate-50 dark:bg-slate-800/50 rounded-[1.5rem] border border-slate-100 dark:border-slate-700 shadow-sm gap-4">
          <div className="space-y-1">
            <Typography className="text-base font-bold text-slate-800 dark:text-slate-200" variant="h4">
              {t("settings.storage.use_custom", "Use Custom Storage Bucket")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
              {t("settings.storage.use_custom_desc", "Store files in your own AWS S3 or MinIO instance instead of the global server.")}
            </Typography>
          </div>
          <label className="relative inline-flex items-center cursor-pointer flex-shrink-0" htmlFor="use-custom-storage">
            <span className="sr-only">{t("settings.storage.use_custom", "Use Custom Storage Bucket")}</span>
            <input 
              checked={isCustom} 
              className="sr-only peer" 
              id="use-custom-storage"
              onChange={(e) => {
                setIsCustom(e.target.checked);
                setTestResult(null);
              }} 
              type="checkbox" 
            />
            <div className="w-14 h-8 bg-slate-200 peer-focus:outline-none rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[4px] after:left-[4px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all dark:border-slate-600 peer-checked:bg-primary-600"></div>
          </label>
        </div>

        {/* Custom Storage Fields */}
        {isCustom ? (
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
                  onChange={(e) => { setEndpointUrl(e.target.value); }}
                  placeholder="https://s3.amazonaws.com or https://play.min.io:9000"
                  type="url"
                  value={endpointUrl}
                />
              </div>
            </div>

            <div className="space-y-3">
              <label className={LABEL_BASE}>{t("settings.storage.bucket", "Bucket Name")}</label>
              <div className="relative">
                <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Database size={18} /></div>
                <input
                  className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                  onChange={(e) => { setBucketName(e.target.value); }}
                  placeholder="my-company-assets"
                  type="text"
                  value={bucketName}
                />
              </div>
            </div>

            <div className="space-y-3">
              <label className={LABEL_BASE}>{t("settings.storage.region", "Region")}</label>
              <div className="relative">
                <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Server size={18} /></div>
                <input
                  className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                  onChange={(e) => { setRegion(e.target.value); }}
                  placeholder="us-east-1"
                  type="text"
                  value={region}
                />
              </div>
            </div>

            <div className="space-y-3">
              <label className={LABEL_BASE}>{t("settings.storage.access_key", "Access Key")}</label>
              <div className="relative">
                <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Key size={18} /></div>
                <input
                  className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                  onChange={(e) => { setAccessKey(e.target.value); }}
                  placeholder="AKIAIOSFODNN7EXAMPLE"
                  type="text"
                  value={accessKey}
                />
              </div>
            </div>

            <div className="space-y-3">
              <label className={LABEL_BASE}>{t("settings.storage.secret_key", "Secret Key")}</label>
              <div className="relative">
                <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400"><Shield size={18} /></div>
                <input
                  className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                  onChange={(e) => { setSecretKey(e.target.value); }}
                  placeholder={config ? "•••••••••••••••• (Leave blank to keep existing)" : "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"}
                  type="password"
                  value={secretKey}
                />
              </div>
            </div>

            <div className="col-span-full pt-2">
              <label className="flex items-center gap-3 cursor-pointer p-4 bg-slate-50 dark:bg-slate-800/50 rounded-[1.5rem] border border-slate-100 dark:border-slate-700 hover:border-primary-200 transition-colors" htmlFor="use-path-style">
                <span className="sr-only">{t("settings.storage.path_style", "Use Path-Style URLs")}</span>
                <input
                  checked={usePathStyle}
                  className="w-5 h-5 text-primary-600 rounded border-slate-300 focus:ring-primary-500"
                  id="use-path-style"
                  onChange={(e) => { setUsePathStyle(e.target.checked); }}
                  type="checkbox"
                />
                <div>
                  <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200">{t("settings.storage.path_style", "Use Path-Style URLs")}</Typography>
                  <Typography className="text-[10px] text-slate-500 dark:text-slate-400" variant="p">{t("settings.storage.path_style_desc", "Enable this if you are using MinIO or a local S3 clone. Disable for AWS S3 (Virtual Hosted-Style).")}</Typography>
                </div>
              </label>
            </div>

            {/* Test Connection */}
            <div className="col-span-full flex items-center gap-4 mt-6 p-6 bg-slate-50 dark:bg-slate-800/30 rounded-[1.5rem] border border-slate-100 dark:border-slate-800">
              <Button
                disabled={isTesting || upsertMutation.isPending}
                loading={isTesting}
                onClick={handleTestConnection}
                variant="outline"
              >
                {t("settings.storage.test_btn", "Test Connection").toUpperCase()}
              </Button>
              
              {testResult === "success" && (
                <span className="flex items-center text-sm font-bold text-emerald-600 dark:text-emerald-400 gap-2 animate-in fade-in bg-emerald-50 dark:bg-emerald-950/30 px-4 py-2 rounded-xl">
                  <CheckCircle2 className="w-5 h-5" />
                  {t("settings.storage.test_success", "Connection successful!")}
                </span>
              )}
              {testResult === "error" && (
                <span className="flex items-center text-sm font-bold text-rose-600 dark:text-rose-400 gap-2 animate-in fade-in bg-rose-50 dark:bg-rose-950/30 px-4 py-2 rounded-xl">
                  <XCircle className="w-5 h-5" />
                  {t("settings.storage.test_error", "Connection failed. Check credentials.")}
                </span>
              )}
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

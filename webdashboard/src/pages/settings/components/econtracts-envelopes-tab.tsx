import { AlertCircle, Eye, FileText, Search } from "lucide-react";
import React, { useState } from "react";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import EmptyState from "@/components/ui/empty-state";
import Modal from "@/components/ui/modal";
import { EContractEnvelope } from "@/lib/api/econtracts";
import { cn } from "@/lib/utils";

interface EContractsEnvelopesTabProps {
  envelopes: EContractEnvelope[];
  isLoading: boolean;
}

export default function EContractsEnvelopesTab({
  envelopes,
  isLoading,
}: EContractsEnvelopesTabProps) {
  const [selectedEnvelopeForLog, setSelectedEnvelopeForLog] = useState<EContractEnvelope | null>(null);

  // Filters for Envelope dashboard
  const [envFilterSearch, setEnvFilterSearch] = useState("");
  const [envFilterProvider, setEnvFilterProvider] = useState("all");
  const [envFilterStatus, setEnvFilterStatus] = useState("all");

  const filteredEnvelopes = envelopes.filter((env) => {
    const matchesSearch =
      env.external_envelope_id.toLowerCase().includes(envFilterSearch.toLowerCase()) ||
      env.id.toLowerCase().includes(envFilterSearch.toLowerCase());
    const matchesProvider = envFilterProvider === "all" || env.provider_type === envFilterProvider;
    const matchesStatus = envFilterStatus === "all" || env.status === envFilterStatus;
    return matchesSearch && matchesProvider && matchesStatus;
  });

  return (
    <div className="space-y-6">
      <EnvelopeFilters
        envFilterProvider={envFilterProvider}
        envFilterSearch={envFilterSearch}
        envFilterStatus={envFilterStatus}
        setEnvFilterProvider={setEnvFilterProvider}
        setEnvFilterSearch={setEnvFilterSearch}
        setEnvFilterStatus={setEnvFilterStatus}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-8 text-center text-xs text-slate-500 animate-pulse">Loading envelopes...</div>
        ) : filteredEnvelopes.length === 0 ? (
          <EmptyState
            description="No signing envelope transactions match the filter criteria."
            icon={<FileText size={40} />}
            title="No Envelopes Found"
          />
        ) : (
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50 dark:bg-slate-800/10">
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                  External ID
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                  Provider
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                  Status
                </th>
                <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                  Date Generated
                </th>
                <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800"></th>
              </tr>
            </thead>
            <tbody>
              {filteredEnvelopes.map((env) => (
                <tr
                  className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors"
                  key={env.id}
                >
                  <td className="px-8 py-6 text-xs font-mono font-bold text-slate-800 dark:text-slate-200">
                    {env.external_envelope_id}
                  </td>
                  <td className="px-8 py-6 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-350">
                    {env.provider_type}
                  </td>
                  <td className="px-8 py-6">
                    <span
                      className={cn(
                        "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 rounded-full border shadow-sm",
                        env.status === "completed"
                          ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
                          : env.status === "sent"
                          ? "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800"
                          : "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800/50 dark:text-slate-400 dark:border-slate-750"
                      )}
                    >
                      {env.status}
                    </span>
                  </td>
                  <td className="px-8 py-6 text-xs font-mono text-slate-500 dark:text-slate-400">
                    {new Date(env.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-8 py-6 text-right">
                    <Button
                      className="flex items-center gap-1 text-[10px] font-bold text-slate-500 hover:text-primary-600 px-3 py-1.5 rounded-xl border border-slate-200 dark:border-slate-850 hover:bg-slate-50 transition-colors"
                      onClick={() => {
                        setSelectedEnvelopeForLog(env);
                      }}
                      size="sm"
                      variant="ghost"
                    >
                      <Eye className="mr-1" size={12} /> WEBHOOK LOGS
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <WebhookLogsModal
        envelope={selectedEnvelopeForLog}
        onClose={() => {
          setSelectedEnvelopeForLog(null);
        }}
      />
    </div>
  );
}

interface EnvelopeFiltersProps {
  envFilterProvider: string;
  envFilterSearch: string;
  envFilterStatus: string;
  setEnvFilterProvider: (val: string) => void;
  setEnvFilterSearch: (val: string) => void;
  setEnvFilterStatus: (val: string) => void;
}

function EnvelopeFilters({
  envFilterProvider,
  envFilterSearch,
  envFilterStatus,
  setEnvFilterProvider,
  setEnvFilterSearch,
  setEnvFilterStatus,
}: EnvelopeFiltersProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-stretch md:items-center justify-between gap-4">
      <div className="relative flex-1 max-w-md">
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" size={16} />
        <input
          aria-label="Search envelopes by ID"
          className="w-full h-11 pl-11 pr-4 bg-slate-50 border border-slate-200 rounded-2xl text-xs font-semibold tracking-tight text-slate-700 dark:text-slate-200 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all dark:bg-slate-850 dark:border-slate-800"
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
            setEnvFilterSearch(e.target.value);
          }}
          placeholder="Search envelopes by ID..."
          type="text"
          value={envFilterSearch}
        />
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <div className="w-40">
          <label className="sr-only" htmlFor="provider-filter">
            Filter by Provider
          </label>
          <Combobox
            id="provider-filter"
            onChange={setEnvFilterProvider}
            options={[
              { value: "all", label: "All Providers" },
              { value: "smartca", label: "VN SmartCA" },
              { value: "hsm", label: "Corporate HSM" },
              { value: "fpt", label: "FPT CeCA" },
              { value: "docusign", label: "DocuSign" },
              { value: "mock", label: "Mock Sandbox" },
            ]}
            placeholder="All Providers"
            value={envFilterProvider}
          />
        </div>

        <div className="w-40">
          <label className="sr-only" htmlFor="status-filter">
            Filter by Status
          </label>
          <Combobox
            id="status-filter"
            onChange={setEnvFilterStatus}
            options={[
              { value: "all", label: "All Statuses" },
              { value: "draft", label: "Draft" },
              { value: "sent", label: "Sent" },
              { value: "completed", label: "Completed" },
              { value: "voided", label: "Voided" },
            ]}
            placeholder="All Statuses"
            value={envFilterStatus}
          />
        </div>
      </div>
    </div>
  );
}

interface WebhookLogsModalProps {
  envelope: EContractEnvelope | null;
  onClose: () => void;
}

function WebhookLogsModal({ envelope, onClose }: WebhookLogsModalProps) {
  return (
    <Modal
      icon={<AlertCircle size={24} />}
      isOpen={Boolean(envelope)}
      onClose={onClose}
      subtitle={`Audit events & incoming webhook responses for ${envelope?.external_envelope_id}`}
      title="Webhooks & Audit Trail"
    >
      {!!envelope && (
        <div className="space-y-6 pt-4">
          <div className="bg-slate-950 p-6 rounded-3xl font-mono text-[11px] text-emerald-400 overflow-x-auto select-all max-h-96 shadow-inner">
            <pre>
              {JSON.stringify(
                envelope.audit_trail_metadata ?? { message: "No callback metadata captured" },
                null,
                2
              )}
            </pre>
          </div>
          <div className="flex items-center justify-end pt-4 border-t border-slate-100 dark:border-slate-800">
            <Button onClick={onClose} type="button" variant="outline">
              CLOSE
            </Button>
          </div>
        </div>
      )}
    </Modal>
  );
}

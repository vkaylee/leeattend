import PageHeader from "@/components/ui/PageHeader";
import { CreditCard } from "lucide-react";

export default function WorkspaceBillingPage() {
  return (
    <div className="space-y-10 animate-in fade-in duration-500">
      <PageHeader
        title="Billing & Subscription"
        subtitle="Manage your subscription plan and invoices"
      />
      
      <div className="bg-slate-50 dark:bg-slate-800/50 p-16 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex flex-col items-center justify-center text-center">
        <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-3xl flex items-center justify-center text-emerald-500 shadow-xl mb-6">
          <CreditCard size={40} />
        </div>
        <h3 className="text-xl font-bold text-slate-800 dark:text-slate-200 mb-2">Billing Coming Soon</h3>
        <p className="text-sm font-medium text-slate-500 max-w-md">
          Billing and subscription management features will be integrated once the commercial version is released.
        </p>
      </div>
    </div>
  );
}

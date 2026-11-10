import { 
  Calculator, 
  Plus, 
  Settings2,
  Calendar,
  AlertCircle
} from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { OvertimePolicy } from '@/lib/api/types';

import AssignOvertimePolicyModal from './assign-overtime-policy-modal';
import { useOvertimePolicies, useOvertimePolicy, useCreateOvertimePolicy } from './hooks/use-overtime-policies';
import OvertimeRuleEditor from './overtime-rule-editor';

function OvertimePolicyCard({ policy}: { policy: OvertimePolicy}) {
  const { data: detail, isLoading } = useOvertimePolicy(policy.id);

  return (
    <div className='bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl p-6 shadow-sm hover:shadow-md transition-all group'>
      <div className='flex items-center justify-between mb-4'>
        <div className='flex items-center gap-4'>
          <div className='w-12 h-12 rounded-2xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors'>
            <Settings2 size={24} />
          </div>
          <div>
            <Typography className='font-bold text-slate-700 dark:text-slate-200 text-lg' variant="h3">{policy.name}</Typography>
            <Typography className='text-sm text-slate-500 dark:text-slate-400' variant="p">{policy.description}</Typography>
          </div>
        </div>
      </div>
      
      {isLoading ? (
        <Skeleton className='h-20 bg-slate-50 dark:bg-slate-800 rounded-2xl' />
      ) : (
        <OvertimeRuleEditor policyId={policy.id} rules={detail ? detail[1] : []}  />
      )}
    </div>
  );
}

function OvertimePolicyForm({
  onCancel,
  onSave,
  isPending
}: {
  isPending: boolean;
  onCancel: () => void;
  onSave: (data: { name: string; description: string }) => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  return (
    <div className='bg-slate-50 dark:bg-slate-800 rounded-3xl p-6 border border-slate-200 dark:border-slate-700 animate-in fade-in slide-in-from-top-4 duration-300'>
      <div className='grid grid-cols-1 md:grid-cols-2 gap-6'>
        <FormField
          id="overtime-policy-name"
          label={t('common.name')}
          onChange={(e) => { setName(e.target.value); }}
          placeholder={t('payroll.overtime_policy_name_placeholder', { defaultValue: 'e.g. Standard Overtime' })}
          type='text'
          value={name}
        />
        <FormField
          id="overtime-policy-description"
          label={t('common.description')}
          onChange={(e) => { setDescription(e.target.value); }}
          placeholder={t('payroll.overtime_policy_desc_placeholder', { defaultValue: 'e.g. Night and Weekend rates' })}
          type='text'
          value={description}
        />
      </div>
      <div className='flex justify-end gap-3 mt-6'>
        <Button className='px-6 py-2 rounded-xl text-sm font-bold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all' onClick={onCancel} variant="ghost">
          {t('common.cancel', { defaultValue: 'Cancel' })}
        </Button>
        <Button className='flex items-center gap-2 bg-primary-600 text-white px-8 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all disabled:opacity-50'
          disabled={!name || isPending}
          onClick={() => { onSave({ name, description }); }}
          variant="primary"
        >
          {t('common.save')}
        </Button>
      </div>
    </div>
  );
}

function OvertimePolicySkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-2">
          <Skeleton className="h-8 w-48 bg-slate-200 dark:bg-slate-800 rounded-xl" />
          <Skeleton className="h-4 w-96 bg-slate-200/60 dark:bg-slate-800/60 rounded-lg" />
        </div>
        <div className="flex gap-3">
          <Skeleton className="h-10 w-32 bg-slate-200 dark:bg-slate-800 rounded-xl" />
          <Skeleton className="h-10 w-32 bg-slate-200 dark:bg-slate-800 rounded-xl" />
        </div>
      </div>
      <div className="grid grid-cols-1 gap-4">
        {[1, 2].map((i) => (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-6 shadow-sm space-y-4" key={i}>
            <div className="flex items-center gap-4">
              <Skeleton className="w-12 h-12 rounded-2xl bg-slate-200 dark:bg-slate-800" />
              <div className="space-y-2">
                <Skeleton className="h-5 w-40 bg-slate-200 dark:bg-slate-800 rounded-lg" />
                <Skeleton className="h-4 w-60 bg-slate-200/60 dark:bg-slate-800/60 rounded-lg" />
              </div>
            </div>
            <Skeleton className="h-20 w-full bg-slate-100/50 dark:bg-slate-800/30 rounded-2xl animate-pulse" />
          </div>
        ))}
      </div>
    </div>
  );
}

export default function OvertimePolicyManager() {
  const { t } = useTranslation();
  const [isCreating, setIsCreating] = useState(false);
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);

  const { data: policies = [], isLoading } = useOvertimePolicies();

  const createMutation = useCreateOvertimePolicy({
    onSuccess: () => {
      setIsCreating(false);
    },
  });

  if (isLoading) return <OvertimePolicySkeleton />;

  return (
    <div className='space-y-6'>
      <div className='flex items-center justify-between'>
        <div>
          <Typography className='text-xl font-bold text-slate-700 dark:text-slate-200 flex items-center gap-2' variant="h2">
            <Calculator className='text-primary-600 dark:text-primary-400' size={20} />
            {t('payroll.overtime_policies')}
          </Typography>
          <Typography className='text-sm text-slate-500 dark:text-slate-400 mt-1' variant="p">{t('payroll.overtime_policies_desc')}</Typography>
        </div>
        <div className='flex items-center gap-3'>
          <Button className='flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-4 py-2 rounded-xl text-sm font-bold hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95'
            onClick={() => { setIsAssignModalOpen(true); }}
            variant="ghost"
          >
            <Calendar size={16} />
            {t('payroll.assign_policy')}
          </Button>
          <Button className='flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95'
            onClick={() => { setIsCreating(!isCreating); }}
            variant="ghost"
          >
            <Plus size={16} />
            {t('common.create')}
          </Button>
        </div>
      </div>

      {isCreating ? (
        <OvertimePolicyForm 
          isPending={createMutation.isPending}
          onCancel={() => { setIsCreating(false); }}
          onSave={(data) => { createMutation.mutate(data); }}
        />
      ) : null}

      <div className='grid grid-cols-1 gap-4'>
        {policies.map((policy: OvertimePolicy) => (
          <OvertimePolicyCard key={policy.id} policy={policy}  />
        ))}

        {policies.length === 0 && !isCreating ? (
          <div className='flex flex-col items-center justify-center py-12 bg-slate-50 dark:bg-slate-800 rounded-2xl sm:rounded-[2.5rem] border-2 border-dashed border-slate-200 dark:border-slate-700'>
             <AlertCircle className='text-slate-300 dark:text-slate-600 mb-4' size={48} />
             <Typography className='text-slate-500 dark:text-slate-400 font-medium' variant="p">{t('payroll.no_overtime_policies')}</Typography>
             <Button className='mt-4 text-primary-600 dark:text-primary-400 font-bold hover:underline' onClick={() => { setIsCreating(true); }} variant="ghost">{t('payroll.create_first_policy')}</Button>
          </div>
        ) : null}
      </div>

      <AssignOvertimePolicyModal isOpen={isAssignModalOpen} 
      onClose={() => { setIsAssignModalOpen(false); }}  />
    </div>
  );
}

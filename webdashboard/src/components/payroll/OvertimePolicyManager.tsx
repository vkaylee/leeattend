import React, { useState } from 'react';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  Calculator, 
  Plus, 
  Settings2,
  Calendar,
  AlertCircle
} from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { useToast } from '@/components/ui/Toast';

import { fetchOvertimePolicies, createOvertimePolicy, fetchOvertimePolicy } from '@/lib/api/payroll';

import AssignOvertimePolicyModal from './AssignOvertimePolicyModal';
import OvertimeRuleEditor from './OvertimeRuleEditor';

interface OvertimePolicyManagerProps {
  wsId: string;
}

function OvertimePolicyCard({ policy, wsId }: { policy: any, wsId: string }) {
  const { data: detail, isLoading } = useQuery({
    queryKey: ['overtime-policy', policy.id],
    queryFn: () => fetchOvertimePolicy(wsId, policy.id),
  });

  return (
    <div className='bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl p-6 shadow-sm hover:shadow-md transition-all group'>
      <div className='flex items-center justify-between mb-4'>
        <div className='flex items-center gap-4'>
          <div className='w-12 h-12 rounded-2xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors'>
            <Settings2 size={24} />
          </div>
          <div>
            <h3 className='font-bold text-slate-700 dark:text-slate-200 text-lg'>{policy.name}</h3>
            <p className='text-sm text-slate-500 dark:text-slate-400'>{policy.description}</p>
          </div>
        </div>
      </div>
      
      {isLoading ? (
        <div className='h-20 animate-pulse bg-slate-50 dark:bg-slate-800 rounded-2xl'></div>
      ) : (
        <OvertimeRuleEditor wsId={wsId} policyId={policy.id} rules={detail ? detail[1] : []} />
      )}
    </div>
  );
}

export default function OvertimePolicyManager({ wsId }: OvertimePolicyManagerProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isCreating, setIsCreating] = useState(false);
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);

  // Form State
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  const { data: policies = [], isLoading } = useQuery({
    queryKey: ['overtime-policies', wsId],
    queryFn: () => fetchOvertimePolicies(wsId),
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createOvertimePolicy(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['overtime-policies'] });
      setIsCreating(false);
      setName('');
      setDescription('');
      toast.success(t('common.success'));
    },
  });

  if (isLoading) return <div className='p-8 text-center animate-pulse text-slate-500 dark:text-slate-400 font-bold'>Loading policies...</div>;

  return (
    <div className='space-y-6'>
      <div className='flex items-center justify-between'>
        <div>
          <h2 className='text-xl font-bold text-slate-700 dark:text-slate-200 flex items-center gap-2'>
            <Calculator className='text-primary-600 dark:text-primary-400' size={20} />
            {t('payroll.overtime_policies')}
          </h2>
          <p className='text-sm text-slate-500 dark:text-slate-400 mt-1'>{t('payroll.overtime_policies_desc')}</p>
        </div>
        <div className='flex items-center gap-3'>
          <button
            onClick={() => setIsAssignModalOpen(true)}
            className='flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-4 py-2 rounded-xl text-sm font-bold hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95'
          >
            <Calendar size={16} />
            {t('payroll.assign_policy')}
          </button>
          <button
            onClick={() => setIsCreating(!isCreating)}
            className='flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95'
          >
            <Plus size={16} />
            {t('common.create')}
          </button>
        </div>
      </div>

      {isCreating && (
        <div className='bg-slate-50 dark:bg-slate-800 rounded-3xl p-6 border border-slate-200 dark:border-slate-700 animate-in fade-in slide-in-from-top-4 duration-300'>
          <div className='grid grid-cols-1 md:grid-cols-2 gap-6'>
            <div className='space-y-2'>
              <label className='text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('common.name')}</label>
              <input
                type='text'
                value={name}
                onChange={(e) => setName(e.target.value)}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500 outline-none transition-all'
                placeholder='e.g. Standard Overtime'
              />
            </div>
            <div className='space-y-2'>
              <label className='text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('common.description')}</label>
              <input
                type='text'
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500 outline-none transition-all'
                placeholder='e.g. Night and Weekend rates'
              />
            </div>
          </div>
          <div className='flex justify-end gap-3 mt-6'>
            <button onClick={() => setIsCreating(false)} className='px-6 py-2 rounded-xl text-sm font-bold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all'>Cancel</button>
            <button
              onClick={() => createMutation.mutate({ name, description })}
              disabled={!name || createMutation.isPending}
              className='flex items-center gap-2 bg-primary-600 text-white px-8 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all disabled:opacity-50'
            >
              {t('common.save')}
            </button>
          </div>
        </div>
      )}

      <div className='grid grid-cols-1 gap-4'>
        {policies.map((policy: any) => (
          <OvertimePolicyCard key={policy.id} policy={policy} wsId={wsId} />
        ))}

        {policies.length === 0 && !isCreating && (
          <div className='flex flex-col items-center justify-center py-12 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border-2 border-dashed border-slate-200 dark:border-slate-700'>
             <AlertCircle size={48} className='text-slate-300 dark:text-slate-600 mb-4' />
             <p className='text-slate-500 dark:text-slate-400 font-medium'>{t('payroll.no_overtime_policies')}</p>
             <button onClick={() => setIsCreating(true)} className='mt-4 text-primary-600 dark:text-primary-400 font-bold hover:underline'>{t('payroll.create_first_policy')}</button>
          </div>
        )}
      </div>

      <AssignOvertimePolicyModal 
        wsId={wsId} 
        isOpen={isAssignModalOpen} 
        onClose={() => setIsAssignModalOpen(false)} 
      />
    </div>
  );
}

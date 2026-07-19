import { Building2, User, ShieldCheck, Clock, Calendar } from 'lucide-react';
import React from 'react';

import { EmployeeSearch } from '@/components/common/employee-search';
import Button from "@/components/ui/button";
import { Combobox } from '@/components/ui/combobox';
import { Department } from '@/lib/api/types';
import { cn } from '@/lib/utils';

export function AssigneeTypeSelector({
  selectedType,
  setSelectedType,
  setTargetId,
  t
}: {
  selectedType: 'employee' | 'department' | 'workspace';
  setSelectedType: (val: 'employee' | 'department' | 'workspace') => void;
  setTargetId: (val: string) => void;
  t: (key: string) => string;
}) {
  return (
    <div className='space-y-2'>
      <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('payroll.assignee_type')}</label>
      <div className='flex bg-slate-100 dark:bg-slate-700 p-1 rounded-2xl'>
        {[
          { id: 'employee', label: t('common.employee'), icon: <User size={12}/> },
          { id: 'department', label: t('common.department'), icon: <Building2 size={12}/> },
          { id: 'workspace', label: t('common.global'), icon: <ShieldCheck size={12}/> }
        ].map(opt => (
          <Button className={cn(
              'flex-1 flex items-center justify-center gap-2 py-2 rounded-xl text-xs font-bold transition-all',
              selectedType === opt.id ? 'bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm' : 'text-slate-500 dark:text-slate-400'
            )}
            data-testid={`assignee-type-${opt.id}`}
            key={opt.id}
            onClick={() => {
              setSelectedType(opt.id as 'employee' | 'department' | 'workspace');
              setTargetId('');
            }}
            type='button'
            variant="ghost"
          >
            {opt.icon} {opt.label}
          </Button>
        ))}
      </div>
    </div>
  );
}

export function AssigneePicker({
  selectedType,
  targetId,
  setTargetId,
  wsId,
  departments,
  t
}: {
  departments: Department[];
  selectedType: 'employee' | 'department' | 'workspace';
  setTargetId: (val: string) => void;
  t: (key: string) => string;
  targetId: string;
  wsId: string;
}) {
  if (selectedType === 'workspace') return null;

  if (selectedType === 'employee') {
    return (
      <div className='space-y-2'>
        <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('common.select_employee')}</label>
        <EmployeeSearch
          className='bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14'
          data-testid='assign-employee-search'
          departments={departments}
          onChange={(value) => { setTargetId(Array.isArray(value) ? value[0] : value); }}
          placeholder={t('common.select_employee')}
          value={targetId}
          workspaceId={wsId}
        />
      </div>
    );
  }

  return (
    <div className='space-y-2'>
      <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('common.select_department')}</label>
      <Combobox 
        className='bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14'
        data-testid='assign-dept-select'
        id='dept-select'
        onChange={setTargetId}
        options={departments.map(d => ({
          value: d.id,
          label: d.name,
          icon: <Building2 className='text-slate-500 dark:text-slate-400' size={14} />
        }))}
        placeholder={t('common.select_department')}
        value={targetId}
      />
    </div>
  );
}

export function PolicySelector({
  policyId,
  setPolicyId,
  policies,
  t
}: {
  policies: { id: string; name: string }[];
  policyId: string;
  setPolicyId: (val: string) => void;
  t: (key: string) => string;
}) {
  return (
    <div className='space-y-2'>
      <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('payroll.select_policy')}</label>
      <Combobox 
        className='bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14'
        data-testid='assign-policy-select'
        id='policy-select'
        onChange={setPolicyId}
        options={policies.map((p) => ({
          value: p.id,
          label: p.name,
          icon: <Clock className='text-primary-500' size={14} />
        }))}
        placeholder={t('payroll.select_policy_placeholder')}
        value={policyId}
      />
    </div>
  );
}

export function EffectiveFromInput({
  effectiveFrom,
  setEffectiveFrom,
  t
}: {
  effectiveFrom: string;
  setEffectiveFrom: (val: string) => void;
  t: (key: string) => string;
}) {
  return (
    <div className='space-y-2'>
      <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('payroll.effective_from')}</label>
      <div className='relative'>
        <Calendar className='absolute left-4 top-1/2 -translate-y-1/2 text-slate-500 dark:text-slate-400' size={16} />
        <input 
          className='w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl pl-12 pr-4 py-3 text-sm outline-none focus:ring-2 focus:ring-primary-500 font-bold'
          data-testid='assign-effective-from'
          onChange={e => { setEffectiveFrom(e.target.value); }} 
          required
          type='date'
          value={effectiveFrom}
        />
      </div>
    </div>
  );
}

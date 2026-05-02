import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  Calendar, 
  ShieldCheck,
  Building2,
  User,
  Save,
  Clock
} from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { EmployeeSearch } from '@/components/common/EmployeeSearch';
import { Combobox } from '@/components/ui/Combobox';
import Modal from '@/components/ui/Modal';
import { useToast } from '@/components/ui/Toast';
import { fetchDepartments } from '@/lib/api/departments';
import { fetchOvertimePolicies, assignOvertimePolicy } from '@/lib/api/payroll';
import { cn } from '@/lib/utils';

interface AssignOvertimePolicyModalProps {
  initialId?: string;
  initialType?: 'employee' | 'department' | 'workspace';
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function AssignOvertimePolicyModal({ 
  wsId, 
  isOpen, 
  onClose,
  initialType = 'employee',
  initialId = ''
}: AssignOvertimePolicyModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [selectedType, setSelectedType] = useState<'employee' | 'department' | 'workspace'>(initialType);
  const [targetId, setTargetId] = useState(initialId);
  const [policyId, setPolicyId] = useState('');
  const [effectiveFrom, setEffectiveFrom] = useState(new Date().toISOString().split('T')[0]);

  const { data: policies = [] } = useQuery({
    queryKey: ['overtime-policies', wsId],
    queryFn: () => fetchOvertimePolicies(wsId),
    enabled: isOpen
  });

  const { data: departments = [] } = useQuery({
    queryKey: ['departments', wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: isOpen && selectedType === 'department'
  });

  const assignMutation = useMutation({
 
    mutationFn: (data: Parameters<typeof assignOvertimePolicy>[1]) => assignOvertimePolicy(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['overtime-policy-assignment'] });
      toast.success(t('common.success'));
      onClose();
    },
 
    onError: (error: Error) => {
      toast.error(error.message || t('common.error'));
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (selectedType !== 'workspace' && !targetId) {
      toast.error(t('common.error'));
      return;
    }
    if (!policyId) {
      toast.error(t('common.error'));
      return;
    }

    assignMutation.mutate({
      employee_id: selectedType === 'employee' ? targetId : undefined,
      department_id: selectedType === 'department' ? targetId : undefined,
      policy_id: policyId,
      effective_from: effectiveFrom
    });
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t('payroll.assign_overtime_policy')}>
      <form className='space-y-6' onSubmit={handleSubmit}>
        <div className='grid grid-cols-1 gap-6'>
          <div className='space-y-2'>
            <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('payroll.assignee_type')}</label>
            <div className='flex bg-slate-100 dark:bg-slate-700 p-1 rounded-2xl'>
              {[
                { id: 'employee', label: t('common.employee'), icon: <User size={12}/> },
                { id: 'department', label: t('common.department'), icon: <Building2 size={12}/> },
                { id: 'workspace', label: t('common.global'), icon: <ShieldCheck size={12}/> }
              ].map(opt => (
                <button
                  className={cn(
                    'flex-1 flex items-center justify-center gap-2 py-2 rounded-xl text-xs font-bold transition-all',
                    selectedType === opt.id ? 'bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm' : 'text-slate-500 dark:text-slate-400'
                  )}
                  data-testid={`assignee-type-${opt.id ?? ""}`}
                  key={opt.id}
                  onClick={() => {
 
                    setSelectedType(opt.id as 'employee' | 'department' | 'workspace');
                    setTargetId('');
                  }}
                  type='button'
                >
                  {opt.icon} {opt.label}
                </button>
              ))}
            </div>
          </div>

          {selectedType === 'employee' && (
            <div className='space-y-2'>
              <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('common.select_employee')}</label>
              <EmployeeSearch 
                className='bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14'
                data-testid='assign-employee-search'
                onChange={(value) => { setTargetId(Array.isArray(value) ? value[0] : value); }}
                placeholder={t('common.select_employee')}
                value={targetId}
                workspaceId={wsId}
              />
            </div>
          )}

          {selectedType === 'department' && (
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
          )}

          <div className='space-y-2'>
            <label className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 ml-1'>{t('payroll.select_policy')}</label>
            <Combobox 
              className='bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14'
              data-testid='assign-policy-select'
 
              id='policy-select'
              onChange={setPolicyId}
              options={policies.map((p: { id: string; name: string }) => ({
                value: p.id,
                label: p.name,
                icon: <Clock className='text-primary-500' size={14} />
              }))}
              placeholder={t('payroll.select_policy_placeholder')}
              value={policyId}
            />
          </div>

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
        </div>

        <div className='flex justify-end gap-3 pt-4'>
          <button className='px-6 py-2 rounded-xl text-sm font-bold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all uppercase tracking-widest' onClick={onClose} type='button'>{t('common.cancel')}</button>
          <button
            className='flex items-center gap-2 bg-primary-600 text-white px-8 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 uppercase tracking-widest'
            data-testid='assign-policy-submit-btn'
            disabled={assignMutation.isPending}
            type='submit'
          >
            <Save size={16} />
            {t('payroll.assign_policy')}
          </button>
        </div>
      </form>
    </Modal>
  );
}
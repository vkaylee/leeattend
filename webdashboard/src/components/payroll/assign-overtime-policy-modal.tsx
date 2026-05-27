import { Save } from 'lucide-react';
import React from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import Modal from '@/components/ui/modal';

import { 
  AssigneeTypeSelector,
  AssigneePicker,
  PolicySelector,
  EffectiveFromInput
} from './assign-overtime-policy-components/assign-overtime-policy-fields';
import { useAssignOvertimePolicy } from './assign-overtime-policy-components/use-assign-overtime-policy';

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

  const {
    selectedType,
    setSelectedType,
    targetId,
    setTargetId,
    policyId,
    setPolicyId,
    effectiveFrom,
    setEffectiveFrom,
    policies,
    departments,
    assignMutation,
    handleSubmit
  } = useAssignOvertimePolicy({ wsId, isOpen, onClose, initialType, initialId });

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t('payroll.assign_overtime_policy')}>
      <form className='space-y-6' onSubmit={handleSubmit}>
        <div className='grid grid-cols-1 gap-6'>
          <AssigneeTypeSelector 
            selectedType={selectedType}
            setSelectedType={setSelectedType}
            setTargetId={setTargetId}
            t={t}
          />

          <AssigneePicker 
            departments={departments}
            selectedType={selectedType}
            setTargetId={setTargetId}
            t={t}
            targetId={targetId}
            wsId={wsId}
          />

          <PolicySelector 
            policies={policies}
            policyId={policyId}
            setPolicyId={setPolicyId}
            t={t}
          />

          <EffectiveFromInput 
            effectiveFrom={effectiveFrom}
            setEffectiveFrom={setEffectiveFrom}
            t={t}
          />
        </div>

        <div className='flex justify-end gap-3 pt-4'>
          <Button className='px-6 py-2 rounded-xl text-sm font-bold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all uppercase tracking-widest' onClick={onClose} type='button' variant="ghost">{t('common.cancel')}</Button>
          <Button className='flex items-center gap-2 bg-primary-600 text-white px-8 py-2 rounded-xl text-sm font-bold hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 uppercase tracking-widest'
            data-testid='assign-policy-submit-btn'
            disabled={assignMutation.isPending}
            type='submit'
            variant="primary"
          >
            <Save size={16} />
            {t('payroll.assign_policy')}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
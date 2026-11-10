import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useToast } from '@/components/ui/toast';
import { useDepartments } from '@/hooks/use-departments';
import { fetchOvertimePolicies, assignOvertimePolicy } from '@/lib/api/payroll';

interface UseAssignOvertimePolicyConfig {
  initialId?: string;
  initialType?: 'employee' | 'department' | 'organization';
  isOpen: boolean;
  onClose: () => void;
}

export function useAssignOvertimePolicy({isOpen,
  onClose,
  initialType = 'employee',
  initialId = ''
}: UseAssignOvertimePolicyConfig) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [selectedType, setSelectedType] = useState<'employee' | 'department' | 'organization'>(initialType);
  const [targetId, setTargetId] = useState(initialId);
  const [policyId, setPolicyId] = useState('');
  const [effectiveFrom, setEffectiveFrom] = useState(new Date().toISOString().split('T')[0]);

  const { data: policies = [] } = useQuery({ queryKey: ['overtime-policies'], queryFn: () => fetchOvertimePolicies(),
  enabled: isOpen });

  const { data: departments = [] } = useDepartments({ enabled: isOpen });

  const assignMutation = useMutation({
    mutationFn: (data: Parameters<typeof assignOvertimePolicy>[0]) => assignOvertimePolicy(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['overtime-policy-assignment'] });
      toast.success(t('common.success'));
      onClose();
    },
    onError: (error: Error) => {
      toast.error(error.message || t('common.error'));
    }
  });

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (selectedType !== 'organization' && !targetId) {
      toast.error(t('common.error'));
      return;
    }
    if (!policyId) {
      toast.error(t('common.error'));
      return;
    }

    assignMutation.mutate({
      assignee_type: selectedType,
      assignee_id: selectedType === 'organization' ? null : targetId,
      policy_id: policyId,
      effective_from: effectiveFrom
    });
  };

  return {
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
  };
}

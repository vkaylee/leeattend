import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useToast } from '@/components/ui/toast';
import { fetchDepartments } from '@/lib/api/departments';
import { fetchOvertimePolicies, assignOvertimePolicy } from '@/lib/api/payroll';

interface UseAssignOvertimePolicyConfig {
  initialId?: string;
  initialType?: 'employee' | 'department' | 'workspace';
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export function useAssignOvertimePolicy({
  wsId,
  isOpen,
  onClose,
  initialType = 'employee',
  initialId = ''
}: UseAssignOvertimePolicyConfig) {
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

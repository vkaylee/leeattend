import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  Plus, 
  Trash2, 
  Save,
  Clock,
  Calendar,
  AlertCircle,
  Hash,
  Percent
} from 'lucide-react';
import { OvertimeRule } from "@/lib/api/types";
import { addOvertimeRule } from '@/lib/api/payroll';
import { cn } from '@/lib/utils';
import { useToast } from '@/components/ui/Toast';

interface OvertimeRuleEditorProps {
  wsId: string;
  policyId: string;
  rules: OvertimeRule[];
}

export default function OvertimeRuleEditor({ wsId, policyId, rules }: OvertimeRuleEditorProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isAdding, setIsAdding] = useState(false);

  // Rule Form State
  const [priority, setPriority] = useState(0);
  const [multiplier, setMultiplier] = useState(1.5);
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');
  const [days, setDays] = useState(127); // Default all days
  const [isHoliday, setIsHoliday] = useState(false);

  const addMutation = useMutation({
    mutationFn: (data: Partial<OvertimeRule>) => addOvertimeRule(wsId, policyId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['overtime-policies'] });
      setIsAdding(false);
      resetForm();
      toast.success(t('common.success'));
    },
  });

  const resetForm = () => {
    setPriority(0);
    setMultiplier(1.5);
    setStartTime('');
    setEndTime('');
    setDays(127);
    setIsHoliday(false);
  };

  const weekdays = [
    { bit: 1, label: 'M' },
    { bit: 2, label: 'T' },
    { bit: 4, label: 'W' },
    { bit: 8, label: 'T' },
    { bit: 16, label: 'F' },
    { bit: 32, label: 'S' },
    { bit: 64, label: 'S' },
  ];

  const toggleDay = (bit: number) => {
    setDays(prev => prev ^ bit);
  };

  return (
    <div className='mt-6 space-y-4 border-t border-slate-100 dark:border-slate-800 pt-6'>
      <div className='flex items-center justify-between'>
        <h4 className='text-sm font-bold uppercase tracking-[0.2em] text-slate-500 dark:text-slate-400'>{t('payroll.rules')}</h4>
        <button
          onClick={() => setIsAdding(!isAdding)}
          className='flex items-center gap-1.5 text-xs font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 transition-colors'
        >
          <Plus size={14} />
          {t('payroll.add_rule')}
        </button>
      </div>

      {isAdding && (
        <div className='bg-slate-50 dark:bg-slate-800/50 rounded-2xl p-4 border border-slate-100 dark:border-slate-800 space-y-4 animate-in zoom-in-95 duration-200'>
          <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
            <div className='space-y-1.5'>
              <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>
                <Hash size={10}/> Priority
              </label>
              <input 
                type='number' 
                value={priority} 
                onChange={e => setPriority(parseInt(e.target.value))}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
              />
            </div>
            <div className='space-y-1.5'>
              <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>
                <Percent size={10}/> Multiplier
              </label>
              <input 
                type='number' 
                step='0.1'
                value={multiplier} 
                onChange={e => setMultiplier(parseFloat(e.target.value))}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
              />
            </div>
            <div className='space-y-1.5'>
              <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>
                <Clock size={10}/> Start
              </label>
              <input 
                type='time' 
                value={startTime} 
                onChange={e => setStartTime(e.target.value)}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
              />
            </div>
            <div className='space-y-1.5'>
              <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>
                <Clock size={10}/> End
              </label>
              <input 
                type='time' 
                value={endTime} 
                onChange={e => setEndTime(e.target.value)}
                className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
              />
            </div>
          </div>

          <div className='flex flex-wrap items-center justify-between gap-4'>
            <div className='flex items-center gap-1.5'>
              {weekdays.map(d => (
                <button
                  key={d.bit}
                  onClick={() => toggleDay(d.bit)}
                  className={cn(
                    'w-7 h-7 rounded-lg text-[10px] font-bold transition-all',
                    (days & d.bit) ? 'bg-primary-600 text-white shadow-sm' : 'bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600'
                  )}
                >
                  {d.label}
                </button>
              ))}
            </div>
            
            <div className='flex items-center gap-3'>
              <label className='flex items-center gap-2 cursor-pointer group'>
                <input 
                  type='checkbox' 
                  checked={isHoliday} 
                  onChange={e => setIsHoliday(e.target.checked)}
                  className='w-4 h-4 rounded border-slate-300 dark:border-slate-600 text-primary-600 dark:text-primary-400 focus:ring-primary-500'
                />
                <span className='text-xs font-bold text-slate-500 dark:text-slate-400 group-hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 transition-colors'>Holiday Only</span>
              </label>
              
              <button
                onClick={() => addMutation.mutate({
                  priority,
                  multiplier,
                  time_start: startTime || undefined,
                  time_end: endTime || undefined,
                  days_of_week: days,
                  is_holiday: isHoliday
                })}
                disabled={addMutation.isPending}
                className='bg-primary-600 text-white px-4 py-2 rounded-xl text-xs font-bold hover:bg-primary-700 transition-all shadow-sm'
              >
                {t('common.save')}
              </button>
            </div>
          </div>
        </div>
      )}

      <div className='space-y-2'>
        {rules.sort((a, b) => b.priority - a.priority).map((rule: OvertimeRule) => (
          <div key={rule.id} className='flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/30 rounded-xl border border-slate-100 dark:border-slate-800/50 hover:bg-white dark:hover:bg-slate-900 transition-all'>
            <div className='flex items-center gap-4'>
              <div className='px-2 py-1 bg-primary-50 dark:bg-primary-950 text-primary-700 dark:text-primary-300 rounded-lg text-[10px] font-bold'>
                {rule.multiplier}x
              </div>
              <div className='flex flex-col'>
                <span className='text-xs font-bold text-slate-700 dark:text-slate-200'>
                  {rule.time_start || '00:00'} - {rule.time_end || '23:59'}
                </span>
                <span className='text-[10px] text-slate-500 dark:text-slate-400 font-medium'>
                  Priority: {rule.priority} | {rule.is_holiday ? 'Holidays Only' : 'Regular Days'}
                </span>
              </div>
            </div>
            {/* Delete button could go here */}
          </div>
        ))}
        {rules.length === 0 && !isAdding && (
          <p className='text-center py-4 text-xs text-slate-500 dark:text-slate-400 font-medium italic'>No rules defined yet.</p>
        )}
      </div>
    </div>
  );
}

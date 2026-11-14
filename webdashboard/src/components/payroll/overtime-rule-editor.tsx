import { useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Plus,
  Clock,
  Hash,
  Percent
} from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import { useToast } from '@/components/ui/toast';
import { Typography } from "@/components/ui/typography";
import { addOvertimeRule } from '@/lib/api/payroll';
import { OvertimeRule } from "@/lib/api/types";
import { createQueryKey } from "@/lib/query-keys";
import { cn } from '@/lib/utils';



interface OvertimeRuleEditorProps {
  policyId: string;
  rules: OvertimeRule[];
}

function OvertimeRuleInputs({
  priority, setPriority,
  multiplier, setMultiplier,
  startTime, setStartTime,
  endTime, setEndTime
}: {
  priority: number; setPriority: (val: number) => void;
  multiplier: number; setMultiplier: (val: number) => void;
  startTime: string; setStartTime: (val: string) => void;
  endTime: string; setEndTime: (val: string) => void;
}) {
  return (
    <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
      <div className='space-y-1.5'>
        <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400' htmlFor='rule-priority'>
          <Hash size={10}/> Priority
        </label>
        <input
          className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
          id='rule-priority'
          onChange={e => { setPriority(parseInt(e.target.value)); }}
          type='number'
          value={priority}
        />
      </div>
      <div className='space-y-1.5'>
        <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400' htmlFor='rule-multiplier'>
          <Percent size={10}/> Multiplier
        </label>
        <input
          className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
          id='rule-multiplier'
          onChange={e => { setMultiplier(parseFloat(e.target.value)); }}
          step='0.1'
          type='number'
          value={multiplier}
        />
      </div>
      <div className='space-y-1.5'>
        <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400' htmlFor='rule-start'>
          <Clock size={10}/> Start
        </label>
        <input
          className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
          id='rule-start'
          onChange={e => { setStartTime(e.target.value); }}
          type='time'
          value={startTime}
        />
      </div>
      <div className='space-y-1.5'>
        <label className='flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400' htmlFor='rule-end'>
          <Clock size={10}/> End
        </label>
        <input
          className='w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 text-xs outline-none focus:ring-2 focus:ring-primary-500'
          id='rule-end'
          onChange={e => { setEndTime(e.target.value); }}
          type='time'
          value={endTime}
        />
      </div>
    </div>
  );
}

function OvertimeRuleForm({
  onSubmit,
  isPending
}: {
  onSubmit: (data: Partial<OvertimeRule>) => void;
  isPending: boolean;
}) {
  const { t } = useTranslation();
  const [priority, setPriority] = useState(0);
  const [multiplier, setMultiplier] = useState(1.5);
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');
  const [days, setDays] = useState(127); // Default all days
  const [isHoliday, setIsHoliday] = useState(false);

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
    setDays(previous => previous ^ bit);
  };

  return (
    <div className='bg-slate-50 dark:bg-slate-800/50 rounded-2xl p-4 border border-slate-100 dark:border-slate-800 space-y-4 animate-in zoom-in-95 duration-200'>
      <OvertimeRuleInputs
        endTime={endTime} multiplier={multiplier} priority={priority} setEndTime={setEndTime} setMultiplier={setMultiplier} setPriority={setPriority} setStartTime={setStartTime} startTime={startTime}
      />

      <div className='flex flex-wrap items-center justify-between gap-4'>
        <div className='flex items-center gap-1.5'>
          {weekdays.map(d => (
            <Button className={cn(
                'w-7 h-7 rounded-lg text-[10px] font-bold transition-all',
                (days & d.bit) ? 'bg-primary-600 text-white shadow-sm' : 'bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600'
              )}
              key={d.bit}
              onClick={() => { toggleDay(d.bit); }}
              variant="ghost"
            >
              {d.label}
            </Button>
          ))}
        </div>

        <div className='flex items-center gap-3'>
          <label className='flex items-center gap-2 cursor-pointer group'>
            <input
              checked={isHoliday}
              className='w-4 h-4 rounded border-slate-300 dark:border-slate-600 text-primary-600 dark:text-primary-400 focus:ring-primary-500'
              onChange={e => { setIsHoliday(e.target.checked); }}
              type='checkbox'
            />
            <span className='text-xs font-bold text-slate-500 dark:text-slate-400 group-hover:text-slate-700 dark:hover:text-slate-200 transition-colors'>Holiday Only</span>
          </label>

          <Button className='bg-primary-600 text-white px-4 py-2 rounded-xl text-xs font-bold hover:bg-primary-700 transition-all shadow-sm'
            disabled={isPending}
            onClick={() => {
              onSubmit({
                priority,
                multiplier: String(multiplier),
                time_start: startTime || undefined,
                time_end: endTime || undefined,
                days_of_week: days,
                is_holiday: isHoliday
              });
            }}
            variant="ghost"
          >
            {t('common.save')}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default function OvertimeRuleEditor({policyId, rules }: OvertimeRuleEditorProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isAdding, setIsAdding] = useState(false);

  const addMutation = useMutation({
    mutationFn: (data: Partial<OvertimeRule>) => {
      if (data.multiplier === undefined) throw new Error("Multiplier is required");
      return addOvertimeRule(policyId, {
        days_of_week: data.days_of_week ?? 127,
        is_holiday: !!data.is_holiday,
        multiplier: data.multiplier,
        priority: data.priority ?? 0,
        time_end: data.time_end,
        time_start: data.time_start,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: createQueryKey('overtime-policies') });
      setIsAdding(false);
      toast.success(t('common.success'));
    },
  });

  return (
    <div className='mt-6 space-y-4 border-t border-slate-100 dark:border-slate-800 pt-6'>
      <div className='flex items-center justify-between'>
        <Typography className='text-sm font-bold uppercase tracking-[0.2em] text-slate-500 dark:text-slate-400' variant="h4">{t('payroll.rules.label')}</Typography>
        <Button className='flex items-center gap-1.5 text-xs font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 transition-colors'
          onClick={() => { setIsAdding(!isAdding); }}
          variant="ghost"
        >
          <Plus size={14} />
          {t('payroll.add_rule')}
        </Button>
      </div>

      {isAdding ? (
        <OvertimeRuleForm
          isPending={addMutation.isPending}
          onSubmit={(data) => { addMutation.mutate(data); }}
        />
      ) : null}

      <div className='space-y-2'>
        {[...rules].sort((a, b) => b.priority - a.priority).map((rule: OvertimeRule) => (
          <div className='flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/30 rounded-xl border border-slate-100 dark:border-slate-800/50 hover:bg-white dark:hover:bg-slate-900 transition-all' key={rule.id}>
            <div className='flex items-center gap-4'>
              <div className='px-2 py-1 bg-primary-50 dark:bg-primary-950 text-primary-700 dark:text-primary-300 rounded-lg text-[10px] font-bold'>
                {rule.multiplier}x
              </div>
              <div className='flex flex-col'>
                <span className='text-xs font-bold text-slate-700 dark:text-slate-200'>
                  {rule.time_start ?? '00:00'} - {rule.time_end ?? '23:59'}
                </span>
                <span className='text-[10px] text-slate-500 dark:text-slate-400 font-medium'>
                  Priority: {rule.priority} | {rule.is_holiday ? 'Holidays Only' : 'Regular Days'}
                </span>
              </div>
            </div>
            {/* Delete button could go here */}
          </div>
        ))}
        {rules.length === 0 && !isAdding ? (
          <Typography className='text-center py-4 text-xs text-slate-500 dark:text-slate-400 font-medium italic' variant="p">No rules defined yet.</Typography>
        ) : null}
      </div>
    </div>
  );
}

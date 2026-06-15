/* eslint-disable */
import { Save, Plus } from "lucide-react";
import React, { useRef, useState } from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

export interface TemplateEditorProps {
  initialContent?: string;
  onSave: (content: string) => void;
}

const AVAILABLE_VARIABLES = [
  { group: "Employee", vars: ["{{employee_name}}", "{{employee_email}}", "{{employee_id_number}}", "{{join_date}}"] },
  { group: "Employment", vars: ["{{job_title}}", "{{department_name}}", "{{manager_name}}", "{{base_salary}}"] },
  { group: "Company", vars: ["{{company_name}}", "{{company_address}}", "{{current_date}}"] }
];

export function TemplateEditor({ initialContent = "", onSave }: TemplateEditorProps) {
  const [content, setContent] = useState(initialContent);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const insertVariable = (variable: string) => {
    if (!textareaRef.current) return;
    const textarea = textareaRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    
    const newContent = content.substring(0, start) + variable + content.substring(end);
    setContent(newContent);
    
    // Move cursor after inserted variable
    setTimeout(() => {
      textarea.selectionStart = textarea.selectionEnd = start + variable.length;
      textarea.focus();
    }, 0);
  };

  return (
    <div className="flex flex-col md:flex-row gap-6 h-full min-h-[600px]">
      {/* Editor Main Pane */}
      <div className="flex-1 flex flex-col bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div className="p-4 border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 flex items-center justify-between">
          <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="h4">
            Template Content
          </Typography>
          <Button onClick={() => { onSave(content); }} size="sm">
            <Save className="w-4 h-4 mr-2" />
            Save Template
          </Button>
        </div>
        <textarea
          className="flex-1 w-full p-6 resize-none focus:outline-none bg-transparent text-slate-700 dark:text-slate-300 font-mono text-sm leading-relaxed"
          onChange={(e) => { setContent(e.target.value); }}
          placeholder="Start typing your template here..."
          ref={textareaRef}
          value={content}
        />
      </div>

      {/* Variables Sidebar */}
      <div className="w-full md:w-80 flex flex-col bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-800 rounded-2xl overflow-hidden shrink-0">
        <div className="p-4 border-b border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
          <Typography className="font-bold text-slate-800 dark:text-slate-200" variant="h4">
            Dynamic Variables
          </Typography>
          <p className="text-xs text-slate-500 mt-1">Click any chip to insert it at your cursor.</p>
        </div>
        <div className="p-4 overflow-y-auto flex-1 flex flex-col gap-6">
          {AVAILABLE_VARIABLES.map((group) => (
            <div key={group.group}>
              <Typography className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-3" variant="p">
                {group.group}
              </Typography>
              <div className="flex flex-wrap gap-2">
                {group.vars.map((variable) => (
                  <Button
                    className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-mono text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 hover:border-primary-200 dark:hover:border-primary-800 transition-colors cursor-pointer text-left shadow-sm h-auto font-normal"
                    key={variable}
                    onClick={() => { insertVariable(variable); }}
                    variant="outline"
                  >
                    <Plus className="w-3 h-3 text-slate-400" />
                    {variable}
                  </Button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

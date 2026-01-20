type TabType = "pending" | "approved" | "rejected";

export default function AttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");
  const toast = useToast();

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["attendance-adjustments", wsId, activeTab],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId!, activeTab),
    enabled: !!wsId,
  });

  const { data: employees = [] } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId!), enabled: !!wsId });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveAttendanceAdjustment(wsId!, data.id, { status: data.status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
      queryClient.invalidateQueries({ queryKey: ["time-history"] });
      toast.success(t("common.success"));
    },
  });

  const downloadTemplate = () => {
    const headers = "empNo,empName,empDt";
    const example = "EMP001,Nguyen Van A,11/01/2026 08:00";
    const blob = new Blob([`\uFEFF${headers}\n${example}`], { type: 'text/csv;charset=utf-8;' });
    const link = document.body.appendChild(document.createElement("a"));
    link.href = URL.createObjectURL(blob);
    link.download = "attendance_template.csv";
    link.click();
    document.body.removeChild(link);
  };

  const getEmployeeName = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? `${emp.first_name} ${emp.last_name}` : "Unknown"; };
  const getEmployeeID = (id: string) => { const emp = employees.find(e => e.id === id); return emp ? emp.employee_id_number : "N/A"; };

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-8 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("attendance_confirm.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("attendance_confirm.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={downloadTemplate} className="flex items-center gap-2 px-5 py-2.5 bg-white border border-slate-200 text-slate-700 rounded-2xl hover:bg-slate-50 transition-all font-black text-[10px] uppercase tracking-widest shadow-sm"><FileDown size={14} /> {t("attendance_confirm.excel_template")}</button>
          <button onClick={() => setIsImportDialogOpen(true)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all font-black text-[10px] uppercase tracking-widest shadow-lg shadow-primary-100 active:scale-95"><Upload size={14} /> {t("attendance_confirm.import_excel")}</button>
        </div>
      </div>

      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button onClick={() => setActiveTab("pending")} className={cn("flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'pending' ? "bg-white text-amber-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Clock size={14} /> {t("common.status_pending")}</button>
        <button onClick={() => setActiveTab("approved")} className={cn("flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'approved' ? "bg-white text-emerald-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><CheckCircle size={14} /> {t("common.status_approved")}</button>
        <button onClick={() => setActiveTab("rejected")} className={cn("flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'rejected' ? "bg-white text-rose-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><XCircle size={14} /> {t("common.status_rejected")}</button>
      </div>

      <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-4 duration-500">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.employee")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("attendance_confirm.table.request_time")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.reason")}</th>
                <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.status")}</th>
                <th className="px-8 py-4 text-right"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {isLoading ? (
                <tr><td colSpan={5} className="px-8 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>
              ) : requests.length === 0 ? (
                <tr><td colSpan={5} className="px-8 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_confirm.messages.no_requests")}</td></tr>
              ) : requests.map((req: any) => (
                <tr key={req.id} className="hover:bg-slate-50/50 transition-colors group">
                  <td className="px-8 py-5">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 bg-primary-50 rounded-xl flex items-center justify-center text-primary-600 font-black text-xs shadow-inner border border-primary-100/50 uppercase">{getEmployeeName(req.employee_id).charAt(0)}</div>
                      <div>
                        <div className="font-black text-slate-900 text-sm tracking-tight uppercase">{getEmployeeName(req.employee_id)}</div>
                        <div className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-0.5">{getEmployeeID(req.employee_id)}</div>
                      </div>
                    </div>
                  </td>
                  <td className="px-8 py-5">
                    <div className="text-xs font-black text-slate-700 uppercase tracking-tight">{formatDate(req.requested_punch_in_time)}</div>
                    <div className="text-[10px] text-slate-400 font-black flex items-center gap-1.5 mt-1 opacity-70 uppercase tracking-tighter">
                      <Clock size={10} /> {formatTime(req.requested_punch_in_time)} - {req.requested_punch_out_time ? formatTime(req.requested_punch_out_time) : "?"}
                    </div>
                    {req.is_break_overtime && <div className="mt-2"><span className="px-2 py-0.5 bg-primary-600 text-white rounded-md text-[8px] font-black uppercase tracking-[0.1em]">{t("attendance_confirm.ot_label")}</span></div>}
                  </td>
                  <td className="px-8 py-5 max-w-xs">
                    <div className="text-[10px] text-slate-500 font-bold leading-relaxed line-clamp-2 italic opacity-80">
                      <MessageSquare className="w-3 h-3 inline mr-1 text-slate-300" /> {req.reason || t("common.no_description")}
                    </div>
                  </td>
                  <td className="px-8 py-5">
                    <span className={cn("px-2.5 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border", 
                      req.status === 'pending' ? "bg-amber-50 text-amber-600 border-amber-100" : 
                      req.status === 'approved' ? "bg-emerald-50 text-emerald-600 border-emerald-100" : 
                      "bg-rose-50 text-rose-600 border-rose-100"
                    )}>
                      {req.status === 'pending' ? t("common.status_pending") : req.status === 'approved' ? t("common.status_approved") : t("common.status_rejected")}
                    </span>
                  </td>
                  <td className="px-8 py-5 text-right">
                    {req.status === "pending" && (
                      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all">
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "rejected" })} className="p-2.5 bg-white text-rose-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.reject")}><XCircle size={20} /></button>
                        <button onClick={() => approveMutation.mutate({ id: req.id, status: "approved" })} className="p-2.5 bg-white text-emerald-400 hover:text-emerald-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.approve")}><CheckCircle size={20} /></button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <AttendanceImportDialog isOpen={isImportDialogOpen} onClose={() => setIsImportDialogOpen(false)} title={t("attendance_confirm.modal.import_title")} endpoint="/attendance/upload" targetType="adjustment" />
    </div>
  );
}

      <AttendanceImportDialog 
        isOpen={isImportDialogOpen}
        onClose={() => setIsImportDialogOpen(false)}
        title={t("attendance_confirm.modal.import_title")}
        endpoint="/attendance/upload"
        targetType="adjustment"
      />
    </div>
  );
}

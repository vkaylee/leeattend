import React from "react";
import { CalendarRange } from "lucide-react";

export default function ShiftsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Ca làm việc</h1>
        <p className="text-slate-500">Thiết lập và gán ca làm việc cho nhân viên.</p>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl p-12 text-center shadow-sm">
        <div className="bg-orange-50 w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-4">
          <CalendarRange className="text-orange-600 w-8 h-8" />
        </div>
        <h3 className="text-lg font-bold text-slate-900 mb-2">Tính năng đang phát triển</h3>
        <p className="text-slate-500 max-w-md mx-auto">
          Hệ thống quản lý ca làm việc (Shift Scheduling) đang được tích hợp. 
          Bạn sẽ sớm có thể thiết lập ca gãy, ca hành chính và gán cho từng nhân viên.
        </p>
      </div>
    </div>
  );
}

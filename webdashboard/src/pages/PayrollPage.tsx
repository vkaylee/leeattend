import React from "react";
import { Banknote } from "lucide-react";

export default function PayrollPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Quản lý Lương</h1>
        <p className="text-slate-500">Tính toán lương và quản lý phiếu lương cho nhân viên.</p>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl p-12 text-center shadow-sm">
        <div className="bg-blue-50 w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-4">
          <Banknote className="text-blue-600 w-8 h-8" />
        </div>
        <h3 className="text-lg font-bold text-slate-900 mb-2">Tính năng đang phát triển</h3>
        <p className="text-slate-500 max-w-md mx-auto">
          Hệ thống tính lương tự động dựa trên bảng công và cấu hình lương đang được hoàn thiện. 
          Vui lòng quay lại sau.
        </p>
      </div>
    </div>
  );
}

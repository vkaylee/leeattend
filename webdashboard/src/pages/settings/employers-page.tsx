import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Briefcase, Plus } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useOrganizationQuery } from "@/hooks/common/use-workspace-query";
import { useToast } from "@/components/ui/toast";
import {
  Employer,
  fetchEmployees,
  fetchEmployers,
  createEmployer,
  updateEmployer,
  deleteEmployer,
  getStandaloneOrganizationId,
  CreateEmployerRequest,
  UpdateEmployerRequest,
} from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";
import { EmployerCard } from "@/pages/settings/components/employer-card";
import { EmployerFormModal } from "@/pages/settings/components/employer-form-modal";

// eslint-disable-next-line complexity
export default function EmployersPage() {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingEmployer, setEditingEmployer] = useState<Employer | null>(null);

  // Queries
  const { data: employers = [], isLoading: isEmployersLoading } = useQuery({
    queryKey: ["employers", wsId],
    queryFn: () => {
      if (!wsId) throw new Error("No workspace ID");
      return fetchEmployers(wsId);
    },
    enabled: !!wsId,
  });

  const { data: employeesData, isLoading: isEmployeesLoading } = useOrganizationQuery({
    queryKey: employeeKeys.lookup(wsId, { permission: "employers-page" }),
    queryFn: (workspaceId) => fetchEmployees(workspaceId, 1, 1000),
  });

  const employees = employeesData?.data ?? [];
  const isLoading = isEmployersLoading || isEmployeesLoading;

  // Mutations
  const createMutation = useMutation({
    mutationFn: (data: CreateEmployerRequest) => {
      if (!wsId) throw new Error("No workspace ID");
      return createEmployer(wsId, data);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.created_success", { defaultValue: "Đã tạo pháp nhân thành công." }));
      void queryClient.invalidateQueries({ queryKey: ["employers", wsId] });
      setIsModalOpen(false);
    },
    onError: (err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(msg);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateEmployerRequest) => {
      if (!editingEmployer) throw new Error("No employer selected for editing");
      if (!wsId) throw new Error("No workspace ID");
      return updateEmployer(wsId, editingEmployer.id, data);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.updated_success", { defaultValue: "Đã cập nhật thông tin pháp nhân." }));
      void queryClient.invalidateQueries({ queryKey: ["employers", wsId] });
      setIsModalOpen(false);
      setEditingEmployer(null);
    },
    onError: (err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(msg);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace ID");
      return deleteEmployer(wsId, id);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.deleted_success", { defaultValue: "Đã xóa pháp nhân thành công." }));
      void queryClient.invalidateQueries({ queryKey: ["employers", wsId] });
    },
    onError: (err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      // Handles constraint failure when active employees are assigned
      toast.error(msg);
    },
  });

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  const handleFormSubmit = async (data: CreateEmployerRequest | UpdateEmployerRequest) => {
    if (editingEmployer) {
      await updateMutation.mutateAsync(data);
    } else {
      await createMutation.mutateAsync(data as CreateEmployerRequest);
    }
  };

  const handleDeleteEmployer = async (id: string) => {
    await deleteMutation.mutateAsync(id);
  };

  const isMutating = createMutation.isPending || updateMutation.isPending || deleteMutation.isPending;

  return (
    <PageContainer>
      <PageHeader
        actions={
          <Button
            data-testid="add-employer-btn"
            icon={<Plus size={18} />}
            onClick={() => {
              setEditingEmployer(null);
              setIsModalOpen(true);
            }}
            variant="ghost"
          >
            {t("settings.employers.add_btn", { defaultValue: "THÊM PHÁP NHÂN" }).toUpperCase()}
          </Button>
        }
        data-testid="employers-title"
        icon={<Briefcase size={24} />}
        subtitle={t("settings.employers.subtitle", { defaultValue: "Quản lý danh sách các pháp nhân/đơn vị chủ quản trong doanh nghiệp" })}
        title={t("settings.employers.title", { defaultValue: "Danh sách pháp nhân" })}
      />

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, index) => (
            <div
              className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80"
              key={index}
            />
          ))
        ) : employers.length === 0 ? (
          <div className="col-span-full">
            <EmptyState
              description={t("settings.employers.messages.empty_desc", { defaultValue: "Vui lòng thêm pháp nhân để phân loại và quản lý thông tin nhân sự." })}
              icon={<Briefcase size={40} />}
              title={t("settings.employers.messages.no_employers", { defaultValue: "Chưa có pháp nhân nào" })}
            />
          </div>
        ) : (
          employers.map((emp) => (
            <EmployerCard
              employees={employees}
              employer={emp}
              key={emp.id}
              onDelete={handleDeleteEmployer}
              onEdit={(e) => {
                setEditingEmployer(e);
                setIsModalOpen(true);
              }}
              t={t}
            />
          ))
        )}
      </div>

      <EmployerFormModal
        editingEmployer={editingEmployer}
        isLoading={isMutating}
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
          setEditingEmployer(null);
        }}
        onSubmit={handleFormSubmit}
      />
    </PageContainer>
  );
}

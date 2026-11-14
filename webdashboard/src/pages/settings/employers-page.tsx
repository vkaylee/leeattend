import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Briefcase, Plus } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import {
  Employer,
  fetchEmployees,
  fetchEmployers,
  createEmployer,
  updateEmployer,
  deleteEmployer,
  CreateEmployerRequest,
  UpdateEmployerRequest,
} from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";
import { EmployerCard } from "@/pages/settings/components/employer-card";
import { EmployerFormModal } from "@/pages/settings/components/employer-form-modal";
import { createQueryKey } from "@/lib/query-keys";



export default function EmployersPage() {
  const { t } = useTranslation();

  const queryClient = useQueryClient();
  const toast = useToast();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingEmployer, setEditingEmployer] = useState<Employer | null>(null);

  // Queries
  const { data: employers = [], isLoading: isEmployersLoading } = useQuery({
    queryKey: createQueryKey("employers"),
    queryFn: () => {
      return fetchEmployers();
    },
  });

  const { data: employeesData, isLoading: isEmployeesLoading } = useOrganizationQuery({
    queryKey: employeeKeys.lookup({ permission: "employers-page" }),
    queryFn: () => fetchEmployees(1, 1000),
  });

  const employees = employeesData?.data ?? [];
  const isLoading = isEmployersLoading || isEmployeesLoading;

  // Mutations
  const createMutation = useMutation({
    mutationFn: (data: CreateEmployerRequest) => {

      return createEmployer(data);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.created_success", { defaultValue: "Đã tạo pháp nhân thành công." }));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("employers") });
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

      return updateEmployer(editingEmployer.id, data);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.updated_success", { defaultValue: "Đã cập nhật thông tin pháp nhân." }));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("employers") });
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

      return deleteEmployer(id);
    },
    onSuccess: () => {
      toast.success(t("settings.employers.messages.deleted_success", { defaultValue: "Đã xóa pháp nhân thành công." }));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("employers") });
    },
    onError: (err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      // Handles constraint failure when active employees are assigned
      toast.error(msg);
    },
  });



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

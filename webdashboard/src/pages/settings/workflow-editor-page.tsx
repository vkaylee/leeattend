import React from "react";

import { GeneralConfigCard } from "./components/workflow-editor/general-config-card";
import { WorkflowEditorHeader } from "./components/workflow-editor/workflow-editor-header";
import { WorkflowStepsTimeline } from "./components/workflow-editor/workflow-steps-timeline";
import { useWorkflowEditor } from "./hooks/use-workflow-editor";

export default function WorkflowEditorPage() {
  const {
    ws,
    workflowId,
    isDetailsLoading,
    form,
    fields,
    append,
    remove,
    move,
    watchSteps,
    handleBack,
    onSubmit,
    isPending,
    permissionOptions,
    roleOptions,
    memberOptions,
  } = useWorkflowEditor();

  const entityType = form.watch("entity_type");

  if (!ws) return null;

  if (workflowId && isDetailsLoading) {
    return (
      <div className="flex items-center justify-center p-12 min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <WorkflowEditorHeader onBack={handleBack} workflowId={workflowId} />

      <form className="grid grid-cols-1 lg:grid-cols-12 gap-8" onSubmit={form.handleSubmit(onSubmit)}>
        {/* Left Column: General Configuration */}
        <div className="lg:col-span-4 space-y-6">
          <GeneralConfigCard
            entityType={entityType}
            errors={form.formState.errors}
            isPending={isPending}
            onBack={handleBack}
            register={form.register}
          />
        </div>

        {/* Right Column: Workflow Steps timeline */}
        <WorkflowStepsTimeline
          append={append}
          control={form.control}
          fields={fields}
          memberOptions={memberOptions}
          move={move}
          permissionOptions={permissionOptions}
          register={form.register}
          remove={remove}
          roleOptions={roleOptions}
          watchSteps={watchSteps}
        />
      </form>
    </div>
  );
}

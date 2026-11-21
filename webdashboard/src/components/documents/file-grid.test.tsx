import { render, screen } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import type { FileRecord } from "@/lib/api/types";

import { FileGrid } from "./file-grid";

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: () => "2026-11-21",
  }),
}));

const file: FileRecord = {
  category: "general",
  created_at: "2026-11-21T00:00:00Z",
  file_name: "policy.pdf",
  file_size: 1024,
  id: "file-1",
  is_active: true,
  is_latest: true,
  mime_type: "application/pdf",
  owner_id: "owner-1",
  owner_type: "organization",
  scan_status: "clean",
  storage_key: "documents/policy.pdf",
  updated_at: "2026-11-21T00:00:00Z",
  version: 1,
};

describe("FileGrid", () => {
  it("renders files without unavailable actions when callbacks are omitted", () => {
    render(<FileGrid files={[file]} />);

    expect(screen.getByText("policy.pdf")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /policy\.pdf/i })).not.toBeInTheDocument();
  });

  it("keeps file actions available when callbacks are provided", () => {
    const onDeleteFile = vi.fn();
    const onDownloadFile = vi.fn();
    const onViewFile = vi.fn();

    render(
      <FileGrid
        files={[file]}
        onDeleteFile={onDeleteFile}
        onDownloadFile={onDownloadFile}
        onViewFile={onViewFile}
      />,
    );

    expect(screen.getAllByRole("button")).toHaveLength(3);
    expect(screen.getByRole("button", { name: /policy\.pdf/i })).toBeInTheDocument();
  });
});

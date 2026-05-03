import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import Modal from "./modal";

describe("Modal Component", () => {
  beforeEach(() => {
    // Reset body style before each test
    document.body.style.overflow = "unset";
  });

  it("should not render when isOpen is false", () => {
    render(
      <Modal isOpen={false} onClose={() => { /* empty */ }}>
        <div>Modal Content</div>
      </Modal>
    );

    expect(screen.queryByText("Modal Content")).not.toBeInTheDocument();
  });

  it("should render content when isOpen is true", () => {
    render(
      <Modal isOpen={true} onClose={() => { /* empty */ }}>
        <div>Modal Content</div>
      </Modal>
    );

    expect(screen.getByText("Modal Content")).toBeInTheDocument();
  });

  it("should render title and subtitle when provided", () => {
    render(
      <Modal 
        isOpen={true} 
        onClose={() => { /* empty */ }} 
        subtitle="Test Subtitle" 
        title="Test Title"
      >
        <div>Content</div>
      </Modal>
    );

    expect(screen.getByText("Test Title")).toBeInTheDocument();
    expect(screen.getByText("Test Subtitle")).toBeInTheDocument();
  });

  it("should call onClose when close button is clicked", () => {
    const onClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={onClose} title="Title">
        <div>Content</div>
      </Modal>
    );

    // The close button is the only button in the header if title is provided
    const closeButton = screen.getByRole("button");
    fireEvent.click(closeButton);

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should call onClose when clicking on the backdrop", () => {
    const onClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={onClose}>
        <div data-testid="content">Content</div>
      </Modal>
    );

    const backdrop = document.querySelector(".modal-backdrop");
    if (!backdrop) throw new Error("Backdrop not found");
    
    fireEvent.click(backdrop);

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should not call onClose when clicking on the modal content", () => {
    const onClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={onClose}>
        <div data-testid="modal-inner">Content</div>
      </Modal>
    );

    const content = screen.getByTestId("modal-inner");
    fireEvent.click(content);

    expect(onClose).not.toHaveBeenCalled();
  });

  it("should apply body overflow hidden when open", () => {
    render(
      <Modal isOpen={true} onClose={() => { /* empty */ }}>
        <div>Content</div>
      </Modal>
    );

    expect(document.body.style.overflow).toBe("hidden");
  });

  it("should reset body overflow when closed", () => {
    const { rerender } = render(
      <Modal isOpen={true} onClose={() => { /* empty */ }}>
        <div>Content</div>
      </Modal>
    );

    expect(document.body.style.overflow).toBe("hidden");

    rerender(
      <Modal isOpen={false} onClose={() => { /* empty */ }}>
        <div>Content</div>
      </Modal>
    );

    expect(document.body.style.overflow).toBe("unset");
  });

  it("should apply correct max-width class", () => {
    render(
      <Modal isOpen={true} maxWidth="xl" onClose={() => { /* empty */ }}>
        <div>Content</div>
      </Modal>
    );

    const modalContent = document.querySelector(".modal-content");
    expect(modalContent).toHaveClass("max-w-xl");
  });
});

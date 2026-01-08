import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import AgentEditModal from './AgentEditModal';
import * as api from '@/lib/api';

// Mock the API
vi.mock('@/lib/api', async () => {
  const actual = await vi.importActual<typeof api>('@/lib/api');
  return {
    ...actual,
    updateAuthorizedAgent: vi.fn(),
  };
});

const mockAgent: api.AuthorizedAgent = {
  id: 'TEST_001',
  name: 'Test Agent',
  token: 'secret123',
  is_trusted: true,
  is_active: true,
  workspace_id: 'ws-123',
  created_at: new Date().toISOString(),
};

describe('AgentEditModal', () => {
  it('renders agent data when open', () => {
    render(
      <AgentEditModal 
        agent={mockAgent} 
        isOpen={true} 
        onClose={() => {}} 
        onSave={() => {}} 
      />
    );

    expect(screen.getByText('Chỉnh sửa Agent')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Test Agent')).toBeInTheDocument();
    expect(screen.getByDisplayValue('secret123')).toBeInTheDocument();
    expect(screen.getByDisplayValue('TEST_001')).toBeDisabled();
  });

  it('calls updateAuthorizedAgent and onSave when form is submitted', async () => {
    const onSave = vi.fn();
    const onClose = vi.fn();
    const updatedAgent = { ...mockAgent, name: 'New Name' };
    
    (api.updateAuthorizedAgent as any).mockResolvedValue(updatedAgent);

    render(
      <AgentEditModal 
        agent={mockAgent} 
        isOpen={true} 
        onClose={onClose} 
        onSave={onSave} 
      />
    );

    const nameInput = screen.getByDisplayValue('Test Agent');
    fireEvent.change(nameInput, { target: { value: 'New Name' } });

    const saveButton = screen.getByText('Lưu thay đổi');
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(api.updateAuthorizedAgent).toHaveBeenCalledWith('TEST_001', expect.objectContaining({
        name: 'New Name',
        token: 'secret123',
        is_trusted: true,
        is_active: true,
      }));
      expect(onSave).toHaveBeenCalledWith(updatedAgent);
      expect(onClose).toHaveBeenCalled();
    });
  });

  it('toggles trust and active states', () => {
    render(
      <AgentEditModal 
        agent={mockAgent} 
        isOpen={true} 
        onClose={() => {}} 
        onSave={() => {}} 
      />
    );

    const trustButton = screen.getByText('Đã duyệt');
    const activeButton = screen.getByText('Hoạt động');

    fireEvent.click(trustButton);
    expect(screen.getByText('Chờ duyệt')).toBeInTheDocument();

    fireEvent.click(activeButton);
    expect(screen.getByText('Tạm dừng')).toBeInTheDocument();
  });

  it('does not render when isOpen is false', () => {
    const { container } = render(
      <AgentEditModal 
        agent={mockAgent} 
        isOpen={false} 
        onClose={() => {}} 
        onSave={() => {}} 
      />
    );

    expect(container.firstChild).toBeNull();
  });
});

-- Shift Swap Requests
-- Allows employees to swap shifts with each other via an approval workflow.

CREATE TABLE shift_swap_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,

    -- Who initiated and who is the target
    requester_id    UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    target_id       UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,

    -- Swap details
    swap_date       DATE NOT NULL,
    requester_shift_id UUID REFERENCES shifts(id) ON DELETE SET NULL,
    target_shift_id    UUID REFERENCES shifts(id) ON DELETE SET NULL,

    -- Status: pending_target, pending_admin, approved, rejected, cancelled
    status          VARCHAR(20) NOT NULL DEFAULT 'pending_admin',
    reason          TEXT,

    -- Tracking
    requested_by_user_id UUID REFERENCES users(id),
    approved_by_user_id  UUID REFERENCES users(id),

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Prevent duplicate swap requests for the same pair on the same date
    CONSTRAINT uq_shift_swap_pair_date UNIQUE (workspace_id, requester_id, target_id, swap_date)
);

CREATE INDEX idx_shift_swap_workspace ON shift_swap_requests(workspace_id);
CREATE INDEX idx_shift_swap_status ON shift_swap_requests(workspace_id, status);
CREATE INDEX idx_shift_swap_date ON shift_swap_requests(swap_date);

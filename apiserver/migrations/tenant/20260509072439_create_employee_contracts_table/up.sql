-- Add migration script here
CREATE TYPE public.contract_type_enum AS ENUM (
    'probation',       -- Thử việc
    'training',        -- Đào tạo/Học việc
    'fixed_term',      -- Xác định thời hạn
    'indefinite'       -- Không xác định thời hạn
);

CREATE TYPE public.contract_status_enum AS ENUM (
    'draft',           -- Nháp (chuẩn bị ký)
    'active',          -- Đang hiệu lực
    'expired',         -- Đã hết hạn
    'terminated'       -- Đã chấm dứt (trước hạn)
);

CREATE TABLE public.employee_contracts (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    contract_number character varying(100),
    contract_type public.contract_type_enum NOT NULL,
    status public.contract_status_enum DEFAULT 'active'::public.contract_status_enum NOT NULL,
    start_date date NOT NULL,
    end_date date,
    document_id uuid REFERENCES public.employee_documents(id) ON DELETE SET NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    
    CONSTRAINT chk_contract_dates CHECK (end_date IS NULL OR start_date <= end_date),
    CONSTRAINT chk_indefinite_end_date CHECK (
        (contract_type = 'indefinite' AND end_date IS NULL) OR
        (contract_type != 'indefinite')
    )
);

-- Indexes for performance
CREATE INDEX idx_employee_contracts_workspace_id ON public.employee_contracts(workspace_id);

CREATE INDEX idx_employee_contracts_employee_id ON public.employee_contracts(employee_id);

CREATE INDEX idx_employee_contracts_status ON public.employee_contracts(status);

CREATE INDEX idx_employee_contracts_dates ON public.employee_contracts(start_date, end_date);
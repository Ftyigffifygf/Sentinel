-- Create data_exports table for tracking export requests
CREATE TABLE IF NOT EXISTS data_exports (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'in_progress', 'completed', 'failed')),
    export_path TEXT,
    error_message TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_data_exports_tenant ON data_exports(tenant_id, requested_at DESC);
CREATE INDEX idx_data_exports_status ON data_exports(status);

-- Enable row-level security for tenant isolation
ALTER TABLE data_exports ENABLE ROW LEVEL SECURITY;

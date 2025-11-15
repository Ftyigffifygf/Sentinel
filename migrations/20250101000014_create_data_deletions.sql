-- Create data_deletions table for tracking deletion requests
CREATE TABLE IF NOT EXISTS data_deletions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'scheduled', 'in_progress', 'completed', 'failed')),
    scheduled_deletion_date TIMESTAMPTZ NOT NULL,
    actual_deletion_date TIMESTAMPTZ,
    error_message TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_data_deletions_tenant ON data_deletions(tenant_id);
CREATE INDEX idx_data_deletions_status ON data_deletions(status, scheduled_deletion_date);

-- Enable row-level security for tenant isolation
ALTER TABLE data_deletions ENABLE ROW LEVEL SECURITY;

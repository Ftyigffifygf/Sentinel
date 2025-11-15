-- Create retention_policies table
CREATE TABLE IF NOT EXISTS retention_policies (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    artifact_retention_days INTEGER NOT NULL DEFAULT 90,
    verdict_retention_days INTEGER NOT NULL DEFAULT 365,
    audit_log_retention_days INTEGER NOT NULL DEFAULT 730,
    endpoint_event_retention_days INTEGER NOT NULL DEFAULT 90,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT retention_policies_tenant_unique UNIQUE (tenant_id)
);

CREATE INDEX idx_retention_policies_tenant ON retention_policies(tenant_id);

-- Enable row-level security for tenant isolation
ALTER TABLE retention_policies ENABLE ROW LEVEL SECURITY;

-- Add last_cleanup_run column to track cleanup job execution
ALTER TABLE retention_policies ADD COLUMN IF NOT EXISTS last_cleanup_run TIMESTAMPTZ;

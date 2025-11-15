-- Create audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_tenant ON audit_logs(tenant_id, created_at DESC);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id, created_at DESC);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Enable row-level security for tenant isolation
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Make audit logs immutable (prevent updates and deletes)
CREATE OR REPLACE RULE audit_logs_no_update AS
    ON UPDATE TO audit_logs
    DO INSTEAD NOTHING;

CREATE OR REPLACE RULE audit_logs_no_delete AS
    ON DELETE TO audit_logs
    DO INSTEAD NOTHING;

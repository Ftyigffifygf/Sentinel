-- Create hash_lists table (allow/deny lists)
CREATE TABLE IF NOT EXISTS hash_lists (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    hash_type VARCHAR(10) NOT NULL CHECK (hash_type IN ('sha256', 'md5', 'ssdeep')),
    hash_value VARCHAR(255) NOT NULL,
    list_type VARCHAR(10) NOT NULL CHECK (list_type IN ('allow', 'deny')),
    reason TEXT,
    threat_classification VARCHAR(50),
    added_by UUID REFERENCES users(id) ON DELETE SET NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT hash_lists_unique UNIQUE (tenant_id, hash_value, list_type)
);

CREATE INDEX idx_hash_lists_lookup ON hash_lists(tenant_id, hash_value, list_type);
CREATE INDEX idx_hash_lists_tenant ON hash_lists(tenant_id);
CREATE INDEX idx_hash_lists_type ON hash_lists(list_type);
CREATE INDEX idx_hash_lists_added_at ON hash_lists(added_at DESC);

-- Enable row-level security for tenant isolation
ALTER TABLE hash_lists ENABLE ROW LEVEL SECURITY;

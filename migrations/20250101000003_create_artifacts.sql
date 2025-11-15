-- Create artifacts table
CREATE TABLE IF NOT EXISTS artifacts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sha256 VARCHAR(64) NOT NULL,
    md5 VARCHAR(32) NOT NULL,
    ssdeep VARCHAR(255),
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    storage_path VARCHAR(500) NOT NULL,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT artifacts_tenant_sha256_unique UNIQUE (tenant_id, sha256)
);

CREATE INDEX idx_artifacts_tenant ON artifacts(tenant_id);
CREATE INDEX idx_artifacts_sha256 ON artifacts(sha256);
CREATE INDEX idx_artifacts_md5 ON artifacts(md5);
CREATE INDEX idx_artifacts_uploaded_at ON artifacts(uploaded_at DESC);
CREATE INDEX idx_artifacts_uploaded_by ON artifacts(uploaded_by);

-- Enable row-level security for tenant isolation
ALTER TABLE artifacts ENABLE ROW LEVEL SECURITY;

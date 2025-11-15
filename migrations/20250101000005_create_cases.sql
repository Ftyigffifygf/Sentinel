-- Create cases table
CREATE TABLE IF NOT EXISTS cases (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL CHECK (status IN ('open', 'investigating', 'closed')),
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE INDEX idx_cases_tenant ON cases(tenant_id);
CREATE INDEX idx_cases_status ON cases(status);
CREATE INDEX idx_cases_severity ON cases(severity);
CREATE INDEX idx_cases_created_by ON cases(created_by);
CREATE INDEX idx_cases_assigned_to ON cases(assigned_to);
CREATE INDEX idx_cases_created_at ON cases(created_at DESC);

-- Create case_artifacts junction table
CREATE TABLE IF NOT EXISTS case_artifacts (
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    artifact_id UUID NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (case_id, artifact_id)
);

CREATE INDEX idx_case_artifacts_case ON case_artifacts(case_id);
CREATE INDEX idx_case_artifacts_artifact ON case_artifacts(artifact_id);

-- Enable row-level security for tenant isolation
ALTER TABLE cases ENABLE ROW LEVEL SECURITY;

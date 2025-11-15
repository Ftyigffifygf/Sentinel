-- Create verdicts table
CREATE TABLE IF NOT EXISTS verdicts (
    id UUID PRIMARY KEY,
    artifact_id UUID NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    verdict VARCHAR(20) NOT NULL CHECK (verdict IN ('clean', 'suspicious', 'malicious')),
    risk_score SMALLINT NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    static_score SMALLINT CHECK (static_score >= 0 AND static_score <= 100),
    behavioral_score SMALLINT CHECK (behavioral_score >= 0 AND behavioral_score <= 100),
    evidence JSONB NOT NULL,
    overridden_by UUID REFERENCES users(id) ON DELETE SET NULL,
    override_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verdicts_artifact ON verdicts(artifact_id);
CREATE INDEX idx_verdicts_tenant ON verdicts(tenant_id);
CREATE INDEX idx_verdicts_verdict ON verdicts(verdict);
CREATE INDEX idx_verdicts_risk_score ON verdicts(risk_score DESC);
CREATE INDEX idx_verdicts_created_at ON verdicts(created_at DESC);

-- Enable row-level security for tenant isolation
ALTER TABLE verdicts ENABLE ROW LEVEL SECURITY;

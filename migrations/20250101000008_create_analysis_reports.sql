-- Create static_analysis_reports table
CREATE TABLE IF NOT EXISTS static_analysis_reports (
    id UUID PRIMARY KEY,
    artifact_id UUID NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    file_type VARCHAR(20),
    imports JSONB,
    sections JSONB,
    yara_matches JSONB,
    strings JSONB,
    entropy_scores JSONB,
    threat_intel_hits JSONB,
    static_score SMALLINT NOT NULL CHECK (static_score >= 0 AND static_score <= 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_static_reports_artifact ON static_analysis_reports(artifact_id);
CREATE INDEX idx_static_reports_tenant ON static_analysis_reports(tenant_id);
CREATE INDEX idx_static_reports_created_at ON static_analysis_reports(created_at DESC);

-- Create behavioral_analysis_reports table
CREATE TABLE IF NOT EXISTS behavioral_analysis_reports (
    id UUID PRIMARY KEY,
    artifact_id UUID NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    execution_time_ms BIGINT,
    file_operations JSONB,
    registry_operations JSONB,
    process_events JSONB,
    network_events JSONB,
    ransomware_indicators JSONB,
    persistence_mechanisms JSONB,
    behavioral_score SMALLINT NOT NULL CHECK (behavioral_score >= 0 AND behavioral_score <= 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_behavioral_reports_artifact ON behavioral_analysis_reports(artifact_id);
CREATE INDEX idx_behavioral_reports_tenant ON behavioral_analysis_reports(tenant_id);
CREATE INDEX idx_behavioral_reports_created_at ON behavioral_analysis_reports(created_at DESC);

-- Enable row-level security for tenant isolation
ALTER TABLE static_analysis_reports ENABLE ROW LEVEL SECURITY;
ALTER TABLE behavioral_analysis_reports ENABLE ROW LEVEL SECURITY;

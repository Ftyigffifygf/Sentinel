-- Create webhook_integrations table
CREATE TABLE IF NOT EXISTS webhook_integrations (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    format VARCHAR(20) NOT NULL CHECK (format IN ('json', 'cef', 'leef')),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    auth_type VARCHAR(20) CHECK (auth_type IN ('none', 'bearer', 'basic', 'api_key')),
    auth_credentials JSONB,
    retry_config JSONB,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ
);

CREATE INDEX idx_webhook_integrations_tenant ON webhook_integrations(tenant_id);
CREATE INDEX idx_webhook_integrations_enabled ON webhook_integrations(enabled);

-- Create webhook_deliveries table for tracking
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY,
    integration_id UUID NOT NULL REFERENCES webhook_integrations(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    verdict_id UUID REFERENCES verdicts(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'success', 'failed', 'retrying')),
    attempt_count SMALLINT NOT NULL DEFAULT 0,
    response_code INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_webhook_deliveries_integration ON webhook_deliveries(integration_id);
CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_created_at ON webhook_deliveries(created_at DESC);

-- Create threat_intel_feeds table
CREATE TABLE IF NOT EXISTS threat_intel_feeds (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    feed_type VARCHAR(50) NOT NULL,
    format VARCHAR(20) NOT NULL CHECK (format IN ('csv', 'json', 'stix')),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    refresh_interval_minutes INTEGER NOT NULL DEFAULT 15,
    last_updated_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_threat_intel_feeds_enabled ON threat_intel_feeds(enabled);
CREATE INDEX idx_threat_intel_feeds_last_updated ON threat_intel_feeds(last_updated_at);

-- Create threat_intel_indicators table
CREATE TABLE IF NOT EXISTS threat_intel_indicators (
    id UUID PRIMARY KEY,
    feed_id UUID NOT NULL REFERENCES threat_intel_feeds(id) ON DELETE CASCADE,
    indicator_type VARCHAR(20) NOT NULL CHECK (indicator_type IN ('hash', 'domain', 'ip', 'url')),
    indicator_value VARCHAR(255) NOT NULL,
    severity VARCHAR(20) CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    metadata JSONB,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT threat_intel_indicators_unique UNIQUE (feed_id, indicator_type, indicator_value)
);

CREATE INDEX idx_threat_intel_indicators_feed ON threat_intel_indicators(feed_id);
CREATE INDEX idx_threat_intel_indicators_lookup ON threat_intel_indicators(indicator_type, indicator_value);
CREATE INDEX idx_threat_intel_indicators_severity ON threat_intel_indicators(severity);

-- Enable row-level security for tenant isolation
ALTER TABLE webhook_integrations ENABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_deliveries ENABLE ROW LEVEL SECURITY;

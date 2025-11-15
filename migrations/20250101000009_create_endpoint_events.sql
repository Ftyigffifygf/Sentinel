-- Create endpoint_events table for TimescaleDB
-- Note: TimescaleDB extension must be enabled before running this migration
-- Run: CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TABLE IF NOT EXISTS endpoint_events (
    time TIMESTAMPTZ NOT NULL,
    tenant_id UUID NOT NULL,
    endpoint_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    process_name VARCHAR(255),
    process_pid INTEGER,
    file_path TEXT,
    registry_key TEXT,
    network_destination VARCHAR(255),
    event_data JSONB,
    severity SMALLINT CHECK (severity >= 0 AND severity <= 100)
);

-- Create hypertable for time-series data (requires TimescaleDB)
-- This will be executed conditionally if TimescaleDB is available
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_extension WHERE extname = 'timescaledb'
    ) THEN
        PERFORM create_hypertable('endpoint_events', 'time', if_not_exists => TRUE);
    END IF;
END $$;

CREATE INDEX idx_endpoint_events_tenant ON endpoint_events(tenant_id, time DESC);
CREATE INDEX idx_endpoint_events_endpoint ON endpoint_events(endpoint_id, time DESC);
CREATE INDEX idx_endpoint_events_type ON endpoint_events(event_type);
CREATE INDEX idx_endpoint_events_severity ON endpoint_events(severity DESC);

-- Enable row-level security for tenant isolation
ALTER TABLE endpoint_events ENABLE ROW LEVEL SECURITY;

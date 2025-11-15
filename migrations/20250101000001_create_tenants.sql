-- Create tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    encryption_key_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    settings JSONB,
    CONSTRAINT tenants_name_unique UNIQUE (name)
);

CREATE INDEX idx_tenants_created_at ON tenants(created_at DESC);

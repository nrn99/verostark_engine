-- Add migration script here
CREATE TABLE IF NOT EXISTS request_audits (
    request_id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    client_ip TEXT NOT NULL, -- Using TEXT for IP as it handles IPv6/v4 strings flexibily, or use INET if strictly pg compliant
    upstream_status SMALLINT NOT NULL,
    latency_ms INT NOT NULL,
    risk_verdict TEXT NOT NULL,
    scrub_count INT NOT NULL,
    scrub_details JSONB NOT NULL,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON request_audits(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_verdict ON request_audits(risk_verdict);

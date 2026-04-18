CREATE SCHEMA IF NOT EXISTS rebound;

CREATE TABLE rebound.instance (
  id text COLLATE "C" PRIMARY KEY,
  status text NOT NULL DEFAULT 'launching',
  region text NOT NULL,
  vm_memory_mb integer NOT NULL,
  fly_app_name text NOT NULL,
  fly_machine_id text,
  tailscale_device_id text,
  tailscale_node_name text NOT NULL,
  tailscale_auth_key text,
  cost_per_sec_usd double precision NOT NULL,
  max_duration_secs integer NOT NULL DEFAULT 21600,
  created_at timestamptz NOT NULL DEFAULT now(),
  started_at timestamptz,
  stopped_at timestamptz,
  stop_reason text,
  error text,
  CONSTRAINT instance_status_check CHECK (status IN ('launching',
    'running', 'stopping', 'stopped', 'failed'))
);

CREATE INDEX idx_instance_status ON rebound.instance (status);

CREATE INDEX idx_instance_created_at ON rebound.instance (created_at);

CREATE INDEX idx_instance_region ON rebound.instance (region);

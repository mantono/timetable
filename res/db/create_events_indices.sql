CREATE INDEX IF NOT EXISTS key_idx ON events(namespace, key);
CREATE INDEX IF NOT EXISTS state_idx ON events(namespace, scheduled_at, state);
CREATE UNIQUE INDEX IF NOT EXISTS single_scheduled_idx ON events(namespace, key) WHERE state = 'SCHEDULED';
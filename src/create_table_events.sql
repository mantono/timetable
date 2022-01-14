CREATE TYPE state AS ENUM('SCHEDULED', 'DISABLED', 'COMPLETED');
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS events(
    id                     UUID                            NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    key                    VARCHAR(128)                    NOT NULL,
    value                  JSON                            NOT NULL DEFAULT '{}'::json,
    idempotence_key        UUID                            NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    namespace              VARCHAR(64)                     NOT NULL,
    state                  state                           NOT NULL DEFAULT 'SCHEDULED',
    created_at             TIMESTAMP WITH TIME ZONE        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    scheduled_at           TIMESTAMP WITH TIME ZONE        NOT NULL
);

CREATE INDEX key_idx ON events(namespace, key);
CREATE INDEX state_idx ON events(namespace, scheduled_at, state);
CREATE UNIQUE INDEX single_scheduled_idx ON events(namespace, key) WHERE state = 'SCHEDULED';

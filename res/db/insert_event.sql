PREPARE insert_event(uuid, text, timestamp, json) AS

INSERT INTO events(key, namespace, scheduled_at, value)
VALUES($1, $2, $3, $4)
RETURNING id, key, value, idempotence_key, namespace, state, created_at, scheduled_at;
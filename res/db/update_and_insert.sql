BEGIN;
-- state, id, key, namespace
-- EXECUTE update_event($1, $2, $3, $4);

UPDATE events
SET state = $1
WHERE id = $2
AND key = $3
AND namespace = $4
AND state <> 'COMPLETED';


-- key, namespace, scheduled_at, value
-- EXECUTE insert_event($3, $4, $5, $6);

INSERT INTO events(key, namespace, scheduled_at, value)
VALUES($3, $4, $5, $6)
RETURNING id, key, value, idempotence_key, namespace, state, created_at, scheduled_at;


COMMIT;
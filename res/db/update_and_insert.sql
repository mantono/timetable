BEGIN;
-- state, id, key, namespace
EXECUTE update_event($1, $2, $3, $4);
-- key, namespace, scheduled_at, value
EXECUTE insert_event($3, $4, $5, $6);
COMMIT;
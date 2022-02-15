BEGIN;
EXECUTE update_event($1, $2, $3, 4)
EXECUTE insert_event($3, $4, €5, $6)
COMMIT;
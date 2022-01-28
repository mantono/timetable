UPDATE events
SET state = $1
WHERE id = $2
AND key = $3
AND namespace = $4
AND state <> 'COMPLETED'
RETURNING *;
SELECT * FROM events
WHERE namespace = $1
AND (key = $2 OR $2 IS NULL)
AND state IN ($3, $4, $5)
AND (scheduled_at >= $6 OR $6 IS NULL)
AND (scheduled_at < $7 OR $7 IS NULL)
ORDER BY scheduled_at ASC
LIMIT $8;
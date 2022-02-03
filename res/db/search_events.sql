SELECT * FROM events
WHERE namespace = $1
AND (key IS NULL OR key = $2)
AND state IN ($3, $4, $5)
AND scheduled_at BETWEEN $6 AND $7
ORDER BY scheduled_at ASC
LIMIT $8;
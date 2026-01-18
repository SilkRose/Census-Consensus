UPDATE Questions
SET
	asked_by = $2
WHERE id = $1;

UPDATE Questions
SET
	claimed_by = $2
WHERE id = $1;

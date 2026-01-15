UPDATE Questions
SET
	response_percent = $2
WHERE id = $1;

UPDATE Questions
SET
	chapter_id = $2
WHERE id = $1;

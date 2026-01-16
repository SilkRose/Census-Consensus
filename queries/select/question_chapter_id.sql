SELECT
	id, text, type, response_percent, asked_by, created_by, claimed_by, chapter_id, chapter_order, date_created
FROM Questions WHERE chapter_id = $1 LIMIT 1;

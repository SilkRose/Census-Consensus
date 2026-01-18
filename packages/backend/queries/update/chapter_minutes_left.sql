UPDATE Chapters
SET
	minutes_left = $2
WHERE id = $1;

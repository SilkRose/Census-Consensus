UPDATE Chapters
SET
	chapter_order = $2
WHERE id = $1;

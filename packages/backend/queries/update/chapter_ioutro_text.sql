UPDATE Chapters
SET
	outro_text = $2
WHERE id = $1;

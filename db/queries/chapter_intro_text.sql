UPDATE Chapters
SET
	intro_text = $2
WHERE id = $1;

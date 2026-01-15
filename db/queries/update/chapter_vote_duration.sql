UPDATE Chapters
SET
	vote_duration = $2
WHERE id = $1;

UPDATE Users
SET
	feedback_private = $2,
	feedback_public = $3
WHERE id = $1;

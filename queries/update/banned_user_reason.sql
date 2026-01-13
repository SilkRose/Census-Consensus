UPDATE Banned_users
SET
	reason = $2
WHERE id = $1;

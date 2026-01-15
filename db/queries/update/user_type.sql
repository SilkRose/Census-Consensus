UPDATE Users
SET
	type = $2
WHERE id = $1;

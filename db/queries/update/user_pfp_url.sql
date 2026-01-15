UPDATE Users
SET
	pfp_url = $2
WHERE id = $1;

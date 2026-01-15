SELECT
	token, user_id, date_created
FROM Tokens WHERE token = $1;
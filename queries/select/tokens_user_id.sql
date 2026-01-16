SELECT
	token, user_id, date_created
FROM Tokens WHERE user_id = $1;

INSERT INTO Tokens
	(token, user_id)
VALUES
	($1, $2)
RETURNING
	token, user_id, date_created;

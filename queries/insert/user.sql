INSERT INTO Users
	(id, name, pfp_url, type, tokens, date_cached)
VALUES
	($1, $2, $3, $4, $5, $6)
ON CONFLICT(id) DO UPDATE SET
	name = EXCLUDED.name,
	pfp_url = EXCLUDED.pfp_url,
	type = EXCLUDED.type,
	tokens = Users.tokens || EXCLUDED.tokens,
	date_cached = now()
RETURNING
	id, name, pfp_url, type AS "user_type: UserType", tokens, date_cached;

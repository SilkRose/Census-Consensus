INSERT INTO Users
	(id, name, pfp_url, type, date_cached)
VALUES
	($1, $2, $3, $4, $5)
ON CONFLICT(id) DO UPDATE SET
	name = EXCLUDED.name,
	pfp_url = EXCLUDED.pfp_url,
	type = EXCLUDED.type,
	date_cached = now()
RETURNING
	id, name, pfp_url, type AS "user_type: UserType", feedback_private, feedback_public, date_cached;

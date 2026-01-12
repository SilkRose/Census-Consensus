INSERT INTO Users
	(id, name, pfp_url, type, token, feedback_private, feedback_public, date_cached)
VALUES
	($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT(id) DO UPDATE SET
	name = EXCLUDED.name,
	pfp_url = EXCLUDED.pfp_url,
	type = EXCLUDED.type,
	token = EXCLUDED.token,
	feedback_private = EXCLUDED.feedback_private,
	feedback_public = EXCLUDED.feedback_public,
	date_cached = now()
RETURNING
	id, name, pfp_url, type AS "user_type: UserType", token, feedback_private, feedback_public, date_cached;

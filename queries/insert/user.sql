INSERT INTO Users
	(id, name, pfp_url, type)
VALUES
	($1, $2, $3, $4)
ON CONFLICT(id) DO UPDATE SET
	name = EXCLUDED.name,
	pfp_url = EXCLUDED.pfp_url,
	type = EXCLUDED.type
RETURNING
	id, name, pfp_url, type AS "user_type: UserType", feedback_private, feedback_public, date_joined;

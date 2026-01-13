SELECT
	id, name, pfp_url, type, banned, feedback_private, feedback_public, date_cached
FROM Users WHERE id = $1 LIMIT 1;

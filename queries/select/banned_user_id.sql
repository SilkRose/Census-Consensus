SELECT
	id, reason, date_banned
FROM Banned_users WHERE id = $1 LIMIT 1;

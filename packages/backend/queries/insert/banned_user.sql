INSERT INTO Banned_users
	(id, reason, date_banned)
VALUES
	($1, $2, $3)
ON CONFLICT(id) DO UPDATE SET
	reason = EXCLUDED.reason,
	date_cached = now()
RETURNING
	id, reason, date_banned;

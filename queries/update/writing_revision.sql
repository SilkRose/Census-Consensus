UPDATE Writings
SET
	latest_revision = $2
WHERE id = $1;

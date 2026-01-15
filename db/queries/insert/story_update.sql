INSERT INTO Story_updates
	(title, short_description, description, views, total_views,
	words, chapters, comments, rating, likes, dislikes)
VALUES
	($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
RETURNING
	title, short_description, description, views, total_views,
	words, chapters, comments, rating, likes, dislikes, date_cached;

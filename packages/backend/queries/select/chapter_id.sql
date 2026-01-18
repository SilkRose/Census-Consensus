SELECT
	id, title, vote_duration, minutes_left, fimfic_ch_id, intro_text, outro_text, chapter_order, date_created
FROM Chapters WHERE id = $1 LIMIT 1;

CREATE TABLE IF NOT EXISTS Settings (
	story_id   integer     NOT NULL PRIMARY KEY,
	population integer     NOT NULL,
	start_time timestamptz NULL
);

CREATE UNIQUE INDEX one_row_only ON Settings ((true));

INSERT INTO Settings
	(story_id, population, start_time)
VALUES
	(552650, 50240000, NULL);

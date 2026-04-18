DROP TABLE IF EXISTS Story_updates;

UPDATE Settings SET start_time = NULL;

ALTER TABLE Chapters DROP CONSTRAINT IF EXISTS Order_minimum;

UPDATE Chapters SET chapter_order = 0 WHERE id = 1;

UPDATE Chapters SET fimfic_ch_id = 1891831 WHERE id = 1;

CREATE TABLE IF NOT EXISTS Votes_complete (
	voter_id     integer     NOT NULL,
	question_id  integer     NOT NULL,
	option_id    text        NOT NULL,
	date_created timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Votes_complete_Users_fk FOREIGN KEY (voter_id)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Votes_complete_Questions_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE,

	CONSTRAINT Votes_complete_pk PRIMARY KEY (voter_id, question_id, option_id)
);

INSERT INTO Votes_complete
	(voter_id, question_id, option_id, date_created)
SELECT
	voter_id, question_id, option_id, date_created
FROM
	Votes;

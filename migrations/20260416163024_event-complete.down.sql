CREATE TABLE IF NOT EXISTS Story_updates (
	title             text        NOT NULL,
	short_description text        NOT NULL,
	description       text        NOT NULL,
	views             integer     NOT NULL,
	total_views       integer     NOT NULL,
	words             integer     NOT NULL,
	chapters          integer     NOT NULL,
	comments          integer     NOT NULL,
	rating            integer     NOT NULL,
	likes             integer     NOT NULL,
	dislikes          integer     NOT NULL,
	date_cached       timestamptz NOT NULL PRIMARY KEY DEFAULT now()
);

DROP TABLE IF EXISTS Votes_complete;

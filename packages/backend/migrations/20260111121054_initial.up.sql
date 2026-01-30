CREATE TYPE user_type AS enum (
	'admin',
	'writer',
	'voter'
);

CREATE TYPE question_type AS enum (
	'multiple_choice',
	'multiselect',
	'scale'
);

CREATE TABLE IF NOT EXISTS Users (
	id               integer     NOT NULL PRIMARY KEY,
	name             text        NOT NULL,
	pfp_url          text        NULL,
	type             user_type   NOT NULL,
	feedback_private text        NULL,
	feedback_public  text        NULL,
	date_last_fetch  timestamptz NOT NULL DEFAULT now(),
	date_joined      timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS Tokens (
	token        text        NOT NULL PRIMARY KEY,
	user_id      integer     NOT NULL,
	user_agent   text        NOT NULL,
	last_seen    timestamptz NOT NULL DEFAULT now(),
	date_created timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Tokens_Users_fk FOREIGN KEY (user_id)
		REFERENCES Users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Banned_users (
	id          integer     NOT NULL PRIMARY KEY,
	reason      text        NOT NULL,
	date_banned timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS Chapter_revisions (
	id                serial      NOT NULL PRIMARY KEY,
	title             text        NOT NULL,
	intro_text        text        NULL,
	outro_text        text        NULL,
	previous_revision integer     NULL,
	created_by        integer     NOT NULL,
	date_created      timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Chapter_revisions_created_by_Users_fk FOREIGN KEY (created_by)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Chapter_revisions_no_self_reference
		CHECK (previous_revision IS NULL OR previous_revision <> id)
);

CREATE TABLE IF NOT EXISTS Chapters (
	id            serial      NOT NULL PRIMARY KEY,
	vote_duration integer     NOT NULL,
	minutes_left  integer     NULL,
	fimfic_ch_id  integer     NULL,
	chapter_order integer     NULL,
	latest_rev    integer     NOT NULL,
	last_edit     timestamptz NOT NULL DEFAULT now(),
	date_created  timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Order_unique UNIQUE (chapter_order),

	CONSTRAINT Order_minimum
		CHECK (chapter_order > 0),

	CONSTRAINT Chapters_latest_rev_fk FOREIGN KEY (latest_rev)
		REFERENCES Chapter_revisions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Question_writings (
	id                serial      NOT NULL PRIMARY KEY,
	question_text     text        NOT NULL,
	option_writing    text        NULL,
	result_writing    text        NULL,
	asked_by          text        NOT NULL,
	created_by        integer     NOT NULL,
	previous_revision integer     NULL,
	date_created      timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Writings_created_by_Users_fk FOREIGN KEY (created_by)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Writings_previous_revision_fk FOREIGN KEY (previous_revision)
		REFERENCES Question_writings (id) ON DELETE SET NULL,

	CONSTRAINT Writings_no_self_reference
		CHECK (previous_revision IS NULL OR previous_revision <> id)
);

CREATE TABLE IF NOT EXISTS Questions (
	id               serial           NOT NULL PRIMARY KEY,
	type             question_type    NOT NULL,
	response_percent double precision NOT NULL,
	created_by       integer          NOT NULL,
	claimed_by       integer          NULL,
	chapter_id       integer          NULL,
	chapter_order    integer          NULL,
	latest_writing   integer          NOT NULL,
	date_created     timestamptz      NOT NULL DEFAULT now(),

	CONSTRAINT Percent_range
		CHECK (response_percent >= 0 AND response_percent <= 100),

	CONSTRAINT Questions_created_by_Users_fk FOREIGN KEY (created_by)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Questions_claimed_by_Users_fk FOREIGN KEY (claimed_by)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Questions_Chapters_fk FOREIGN KEY (chapter_id)
		REFERENCES Chapters (id) ON DELETE CASCADE,

	CONSTRAINT Questions_latest_writing_fk FOREIGN KEY (latest_writing)
		REFERENCES Question_writings (id) ON DELETE CASCADE,

	CONSTRAINT Questions_chapter_order_unique
		UNIQUE (chapter_id, chapter_order)
);

CREATE TABLE IF NOT EXISTS Votes (
	voter_id     integer     NOT NULL,
	question_id  integer     NOT NULL,
	option_id    integer     NOT NULL,
	date_created timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Votes_Users_fk FOREIGN KEY (voter_id)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT Votes_Questions_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE,

	CONSTRAINT Votes_pk PRIMARY KEY (voter_id, question_id, option_id)
);

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

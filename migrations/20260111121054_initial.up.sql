CREATE TYPE user_type AS enum (
	'admin',
	'writer',
	'voter'
);

CREATE TYPE question_type AS enum (
	'boolean',
	'multiple_choice',
	'multiselect',
	'scale'
);

CREATE TYPE question_status AS enum (
	'unclaimed',
	'claimed',
	'in_progress',
	'written'
);

CREATE TYPE completion_status AS enum (
	'incomplete',
	'complete',
	'hiatus',
	'cancelled'
);

CREATE TYPE content_rating AS enum (
	'everyone',
	'teen',
	'mature'
);

CREATE TABLE IF NOT EXISTS Users (
	id          integer     NOT NULL PRIMARY KEY,
	type        user_type   NOT NULL,
	token       text        NOT NULL,
	date_joined timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS Questions (
	id           integer         NOT NULL PRIMARY KEY,
	text         text            NOT NULL,
	type         question_type   NOT NULL,
	status       question_status NOT NULL,
	asked_by     text            NOT NULL,
	created_by   integer         NOT NULL,
	claimed_by   integer         NULL,
	date_created timestamptz     NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS Boolean_answers (
	id            integer     NOT NULL PRIMARY KEY,
	question_id   integer     NOT NULL,
	bool_option   boolean     NOT NULL,
	text          text        NOT NULL,
	date_created  timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Boolean_answers_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Multiple_choice_answers (
	id            integer     NOT NULL PRIMARY KEY,
	question_id   integer     NOT NULL,
	option_number integer     NOT NULL,
	text          text        NOT NULL,
	date_created  timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Multiple_choice_answers_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Multiselect_answers (
	id            integer     NOT NULL PRIMARY KEY,
	question_id   integer     NOT NULL,
	option_number integer     NOT NULL,
	text          text        NOT NULL,
	date_created  timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Multiselect_answers_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Scale_answers (
	id           integer     NOT NULL PRIMARY KEY,
	question_id  integer     NOT NULL,
	scale_start  integer     NOT NULL,
	scale_end    integer     NOT NULL,
	text         text        NOT NULL,
	date_created timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Scale_answers_fk FOREIGN KEY (question_id)
		REFERENCES Questions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Story_updates (
	id                integer           NOT NULL,
	title             text              NOT NULL,
	short_description text              NOT NULL,
	description       text              NOT NULL,
	published         boolean           NOT NULL,
	link              text              NOT NULL,
	cover_url         text              NULL,
	color_hex         char(6)           NOT NULL,
	views             integer           NOT NULL,
	total_views       integer           NOT NULL,
	words             integer           NOT NULL,
	chapters          integer           NOT NULL,
	comments          integer           NOT NULL,
	rating            integer           NOT NULL,
	completion_status completion_status NOT NULL,
	content_rating    content_rating    NOT NULL,
	--tags            text              NOT NULL,
	likes             integer           NOT NULL,
	dislikes          integer           NOT NULL,
	author_id         integer           NOT NULL,
	date_modified     timestamptz       NOT NULL,
	date_updated      timestamptz       NOT NULL,
	date_published    timestamptz       NOT NULL,
	date_cached       timestamptz       NOT NULL DEFAULT now(),

	CONSTRAINT stories_author_id_fk FOREIGN KEY (author_id)
		REFERENCES Users (id) ON DELETE CASCADE,

	CONSTRAINT story_updates_pk PRIMARY KEY (id, date_cached)
);

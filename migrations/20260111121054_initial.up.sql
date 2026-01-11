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

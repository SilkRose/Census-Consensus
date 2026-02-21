CREATE TYPE logo AS enum (
	'census',
	'consensus'
);

CREATE TABLE IF NOT EXISTS Logo_stats (
	id           serial      NOT NULL PRIMARY KEY,
	logo         logo        NOT NULL,
	user_id      integer     NULL,
	ip_addr      text        NULL,
	date_created timestamptz NOT NULL DEFAULT now(),

	CONSTRAINT Logo_stats_Users_fk FOREIGN KEY (user_id)
		REFERENCES Users (id) ON DELETE CASCADE
);

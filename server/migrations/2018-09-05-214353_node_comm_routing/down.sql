
PRAGMA foreign_keys = ON;

ALTER TABLE nodes RENAME TO __nodes_new;

CREATE TABLE nodes (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	name TEXT NOT NULL UNIQUE
);

INSERT INTO nodes (id, public_id, name)
	SELECT id, public_id, name FROM __nodes_new;

DROP TABLE __nodes_new;

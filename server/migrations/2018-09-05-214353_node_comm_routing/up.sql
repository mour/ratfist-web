
PRAGMA foreign_keys = ON;

ALTER TABLE nodes RENAME TO __nodes_old;

CREATE TABLE nodes (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL UNIQUE,
	name TEXT NOT NULL UNIQUE,
	route_type TEXT NOT NULL,
	route_param TEXT
);

INSERT INTO nodes (id, public_id, name, route_type, route_param)
	SELECT id, public_id, name, "serial", "0" FROM __nodes_old;

DROP TABLE __nodes_old;

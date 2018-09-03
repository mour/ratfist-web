
PRAGMA foreign_keys = ON;

CREATE TABLE nodes (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	name TEXT NOT NULL UNIQUE
);

INSERT INTO nodes (public_id, name) VALUES (0, "default_node");


ALTER TABLE sensors RENAME TO __sensors_old;

CREATE TABLE sensors (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	node_id INTEGER NOT NULL,
	type_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
	FOREIGN KEY (type_id) REFERENCES sensor_types(id) ON DELETE RESTRICT,
	UNIQUE (public_id, node_id, type_id)
);

INSERT INTO sensors (id, public_id, node_id, type_id, name)
	SELECT id, public_id, 1, type_id, name FROM __sensors_old;

DROP TABLE __sensors_old;
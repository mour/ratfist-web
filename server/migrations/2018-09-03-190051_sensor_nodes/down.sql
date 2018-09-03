
PRAGMA foreign_keys = ON;

ALTER TABLE sensors RENAME TO __sensors_new;

CREATE TABLE sensors (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	type_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	FOREIGN KEY (type_id) REFERENCES sensor_types(id) ON DELETE RESTRICT,
	UNIQUE (id, public_id, type_id)
);

INSERT INTO sensors (id, public_id, type_id, name)
	SELECT id, public_id, type_id, name FROM __sensors_new;

DROP TABLE __sensors_new;

DROP TABLE nodes;

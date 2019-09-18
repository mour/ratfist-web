-- This file should undo anything in `up.sql`

PRAGMA foreign_keys = ON;

-- Recreate the sensor_types table & fill with data
CREATE TABLE sensor_types (
	id INTEGER PRIMARY KEY NOT NULL,
	name TEXT NOT NULL UNIQUE
);

INSERT INTO sensor_types (id, name) VALUES (0, "pressure");
INSERT INTO sensor_types (id, name) VALUES (1, "temperature");
INSERT INTO sensor_types (id, name) VALUES (2, "humidity");
INSERT INTO sensor_types (id, name) VALUES (3, "light_level");

-- Since we can't alter existing rows, backup old tables, recreate them with new
-- schema, and fill the new ones with existing data.
ALTER TABLE sensors RENAME TO __sensors_old;
ALTER TABLE measurements RENAME TO __measurements_old;

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

CREATE TABLE measurements (
	id INTEGER PRIMARY KEY NOT NULL,
	sensor_id INTEGER NOT NULL,
	value REAL NOT NULL,
	measured_at INTEGER NOT NULL,
	FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE RESTRICT
);

INSERT INTO sensors (id, public_id, node_id, type_id, name)
	SELECT __sensors_old.id, __sensors_old.public_id, __sensors_old.node_id, __sensors_old.sensor_type, __sensors_old.name
		FROM __sensors_old;

INSERT INTO measurements (id, sensor_id, value, measured_at)
	SELECT id, sensor_id, value, measured_at FROM __measurements_old;

-- Drop the old tables now that the data is transfered to the new tables.
DROP TABLE __sensors_old;
DROP TABLE __measurements_old;

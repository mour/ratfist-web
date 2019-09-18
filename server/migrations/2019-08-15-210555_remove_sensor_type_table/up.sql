PRAGMA foreign_keys = ON;

-- Create temporary mapping to the integer values representing the sensor type.
ALTER TABLE sensor_types ADD COLUMN new_type_id INTEGER;
UPDATE sensor_types SET new_type_id = 0 WHERE name = "pressure";
UPDATE sensor_types SET new_type_id = 1 WHERE name = "temperature";
UPDATE sensor_types SET new_type_id = 2 WHERE name = "humidity";
UPDATE sensor_types SET new_type_id = 3 WHERE name = "light_level";

-- Since columns can't be modified, recreate the tables, fill them with data
-- from the old tables, and drop the old tables.
ALTER TABLE sensors RENAME TO __sensors_old;
ALTER TABLE measurements RENAME TO __measurements_old;

CREATE TABLE sensors (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	node_id INTEGER NOT NULL,
	sensor_type INTEGER NOT NULL,
	name TEXT NOT NULL,
	FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
	UNIQUE (public_id, node_id, sensor_type)
);

CREATE TABLE measurements (
	id INTEGER PRIMARY KEY NOT NULL,
	sensor_id INTEGER NOT NULL,
	value REAL NOT NULL,
	measured_at INTEGER NOT NULL,
	FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE RESTRICT
);

INSERT INTO sensors (id, public_id, node_id, sensor_type, name)
	SELECT __sensors_old.id, __sensors_old.public_id, __sensors_old.node_id, sensor_types.new_type_id, __sensors_old.name
		FROM __sensors_old JOIN sensor_types ON (__sensors_old.type_id = sensor_types.id);

INSERT INTO measurements (id, sensor_id, value, measured_at)
	SELECT id, sensor_id, value, measured_at FROM __measurements_old;

DROP TABLE __measurements_old;
DROP TABLE __sensors_old;
DROP TABLE sensor_types;

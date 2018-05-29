-- Your SQL goes here

CREATE TABLE sensor_types (
	id INTEGER PRIMARY KEY NOT NULL,
	name TEXT NOT NULL UNIQUE
);

CREATE TABLE sensors (
	id INTEGER PRIMARY KEY NOT NULL,
	public_id INTEGER NOT NULL,
	type_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	FOREIGN KEY (type_id) REFERENCES sensor_types(id) ON DELETE RESTRICT,
	UNIQUE (public_id, type_id)
);

CREATE TABLE measurements (
	id INTEGER PRIMARY KEY NOT NULL,
	sensor_id INTEGER NOT NULL,
	value REAL NOT NULL,
	measured_at INTEGER NOT NULL,
	FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE RESTRICT
);

INSERT INTO sensor_types (name) VALUES ("temperature");
INSERT INTO sensor_types (name) VALUES ("humidity");
INSERT INTO sensor_types (name) VALUES ("light level");
INSERT INTO sensor_types (name) VALUES ("pressure");

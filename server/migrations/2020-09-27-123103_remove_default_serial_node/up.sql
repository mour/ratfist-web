
PRAGMA foreign_keys = ON;

DELETE FROM nodes WHERE 
	public_id = 0 AND name = "default_node" AND
	route_type = "serial" AND route_param = "0";

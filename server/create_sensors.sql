insert into sensors (type_id, public_id, name) select id, 0, "temp_sensor" from sensor_types where name = "temperature";
insert into sensors (type_id, public_id, name) select id, 1, "humidity_sensor" from sensor_types where name = "humidity";
insert into sensors (type_id, public_id, name) select id, 2, "humidity_sensor" from sensor_types where name = "humidity";
insert into sensors (type_id, public_id, name) select id, 0, "light_level_sensor" from sensor_types where name = "light level";
insert into sensors (type_id, public_id, name) select id, 0, "pressure_sensor" from sensor_types where name = "pressure";

insert into sensors (type_id, public_id, node_id, name) select id, 0, 2, "pressure_sensor" from sensor_types where name = "pressure";
	
select * from sensors;
select * from nodes;

pragma foreign_keys = on;

insert into nodes (public_id, name, route_type, route_param) values (1, "second_node", "serial", "1");
delete from nodes where name = "test_route";

select * from nodes join sensors on nodes.id = sensors.node_id;

insert into measurements (sensor_id, value, measured_at) values (6, 224, cast(strftime("%s", "now") as integer) * 1000000 - 30000000000);

delete from measurements;

select * from measurements;
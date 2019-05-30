use rocket_contrib::json::Json;

use crate::meteo::models::{Measurement, Sensor, SensorType, SensorTypeEnum};
use crate::meteo::{MeteoError, MeteoResponse};

use crate::db::models::Node;

use crate::utils::{DateTimeUtc, IdRange};

use std::borrow::Borrow;
use std::collections::HashMap;

use crate::db::Db;

use diesel::prelude::*;
use diesel::ExpressionMethods;

fn get_measurements(
    db_conn: Db,
    node_ids: IdRange,
    sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    from_time: DateTimeUtc,
    to_time: Option<DateTimeUtc>,
) -> Result<HashMap<u32, HashMap<u32, Vec<(DateTimeUtc, f32)>>>, MeteoError> {
    let node_id_vec = node_ids.into_iter().map(|v| v as i32).collect::<Vec<i32>>();

    let sensor_id_vec = sensor_ids
        .into_iter()
        .map(|v| v as i32)
        .collect::<Vec<i32>>();

    let nodes = {
        use crate::db::schema::nodes::dsl::*;

        nodes
            .filter(public_id.eq_any(node_id_vec))
            .load::<Node>(&*db_conn)
            .map_err(|_| MeteoError)?
    };

    let sensor_type_id = {
        use crate::meteo::schema::sensor_types::dsl::*;

        let sensor_type_str: &str = sensor_type.borrow();
        sensor_types
            .filter(name.eq(sensor_type_str))
            .first::<SensorType>(&*db_conn)
            .map_err(|_| MeteoError)?
            .id
    };

    let sensors = {
        use crate::meteo::schema::sensors::dsl::*;

        Sensor::belonging_to(&nodes)
            .filter(
                public_id
                    .eq_any(sensor_id_vec)
                    .and(type_id.eq(sensor_type_id)),
            )
            .load::<Sensor>(&*db_conn)
            .map_err(|_| MeteoError)?
    };

    let now = DateTimeUtc::now();
    let measurements = {
        use crate::meteo::schema::measurements::dsl::*;

        Measurement::belonging_to(&sensors)
            .order_by(measured_at.asc())
            .filter(measured_at.ge(&from_time))
            .filter(measured_at.le(to_time.as_ref().unwrap_or(&now)))
            .load::<Measurement>(&*db_conn)
            .map_err(|_| MeteoError)?
    };

    let grouped_measurements = measurements.grouped_by(&sensors);

    let grouped_sensors = sensors
        .into_iter()
        .zip(grouped_measurements)
        .grouped_by(&nodes);

    let grouped_nodes: Vec<(Node, Vec<(Sensor, Vec<Measurement>)>)> =
        nodes.into_iter().zip(grouped_sensors).collect();

    let mut output_map = HashMap::new();

    for (node, sensor_group) in grouped_nodes {
        if sensor_group.is_empty() {
            continue;
        }

        let node_map = output_map
            .entry(node.public_id as u32)
            .or_insert_with(HashMap::new);

        for (sensor, measurement_vec) in sensor_group {
            if measurement_vec.is_empty() {
                continue;
            }

            let measurement_pairs = measurement_vec
                .into_iter()
                .map(|m| (m.measured_at, m.value))
                .collect();

            node_map.insert(sensor.public_id as u32, measurement_pairs);
        }
    }

    Ok(output_map)
}

#[get(
    "/<node_ids>/<sensor_type>/<sensor_ids>?<from>&<to>",
    format = "application/json"
)]
pub fn get_stored_values(
    node_ids: IdRange,
    sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, HashMap<u32, Vec<(DateTimeUtc, f32)>>>> {
    Ok(Json(get_measurements(
        db_conn,
        node_ids,
        sensor_type,
        sensor_ids,
        from,
        to,
    )?))
}

#[get("/structure", format = "application/json")]
pub fn get_global_structure(db_conn: Db) -> MeteoResponse<HashMap<u32, HashMap<String, Vec<u32>>>> {
    let nodes = {
        use crate::db::schema::nodes;

        nodes::table
            .load::<Node>(&*db_conn)
            .map_err(|_| MeteoError)?
    };

    let sensor_type_map = {
        use crate::meteo::schema::sensor_types;

        sensor_types::table
            .load::<SensorType>(&*db_conn)
            .map_err(|_| MeteoError)?
            .into_iter()
            .fold(HashMap::new(), |mut m, sensor_type| {
                m.insert(sensor_type.id, sensor_type.name);
                m
            })
    };

    let grouped_sensors: Vec<Vec<Sensor>> = {
        use crate::meteo::schema::sensors::dsl::*;

        Sensor::belonging_to(&nodes)
            .load::<Sensor>(&*db_conn)
            .map_err(|_| MeteoError)?
            .grouped_by(&nodes)
    };

    let nodes_and_sensors: Vec<(Node, Vec<Sensor>)> = nodes.into_iter()
                                .zip(grouped_sensors)
                                .collect();

    let mut output_map = HashMap::new();

    for (node, sensor_vec) in nodes_and_sensors {
        let node_map = output_map
            .entry(node.public_id as u32)
            .or_insert_with(HashMap::new);

        for sensor in sensor_vec {
            node_map
                .entry(sensor_type_map[&sensor.type_id].clone())
                .or_insert_with(Vec::new)
                .push(sensor.public_id as u32);
        }
    }

    Ok(Json(output_map))
}

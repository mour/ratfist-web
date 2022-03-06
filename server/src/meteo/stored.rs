use rocket::serde::json::Json;

use crate::meteo::models::{Measurement, Sensor, SensorTypeEnum};
use crate::meteo::MeteoResponse;

use crate::db::models::Node;

use crate::utils::{DateTimeUtc, IdRange};

use crate::db::Db;

use std::collections::HashMap;
use std::convert::TryInto;

use diesel::prelude::*;
use diesel::ExpressionMethods;

use anyhow::{anyhow, Result};

fn get_measurements(
    db_conn: Db,
    node_id: u32,
    queried_sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    from_time: DateTimeUtc,
    to_time: Option<DateTimeUtc>,
) -> Result<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    let sensor_id_vec = sensor_ids
        .into_iter()
        .map(|v| v as i32)
        .collect::<Vec<i32>>();

    let db_node_id: i32 = node_id.try_into()?;

    let node = {
        use crate::db::schema::nodes::dsl::*;

        nodes
            .filter(public_id.eq(db_node_id))
            .first::<Node>(&*db_conn)
            .map_err(|e| anyhow!("No node with ID {db_node_id} in DB. {e:?}"))?
    };

    let sensors = {
        use crate::meteo::schema::sensors::dsl::*;

        Sensor::belonging_to(&node)
            .filter(
                public_id
                    .eq_any(sensor_id_vec)
                    .and(sensor_type.eq(queried_sensor_type)),
            )
            .load::<Sensor>(&*db_conn)
            .map_err(|e| anyhow!("Error loading sensor info for node ID {db_node_id}. {e:?}"))?
    };

    let now = DateTimeUtc::now();
    let measurements = {
        use crate::meteo::schema::measurements::dsl::*;

        Measurement::belonging_to(&sensors)
            .order_by(measured_at.asc())
            .filter(measured_at.ge(&from_time))
            .filter(measured_at.le(to_time.as_ref().unwrap_or(&now)))
            .load::<Measurement>(&*db_conn)
            .map_err(|e| {
                anyhow!("Error loading measurement info for node ID {db_node_id}. {e:?}")
            })?
    };

    let grouped_measurements = measurements.grouped_by(&sensors);

    let grouped_sensors: Vec<(Sensor, Vec<Measurement>)> =
        sensors.into_iter().zip(grouped_measurements).collect();

    let mut output_map = HashMap::new();

    for (sensor, measurement_vec) in grouped_sensors {
        if measurement_vec.is_empty() {
            continue;
        }

        let measurement_pairs = measurement_vec
            .into_iter()
            .map(|m| (m.measured_at, m.value))
            .collect();

        output_map.insert(sensor.public_id.try_into()?, measurement_pairs);
    }

    Ok(output_map)
}

#[get(
    "/<node_id>/<sensor_type>/<sensor_ids>?<from>&<to>",
    format = "application/json"
)]
pub fn get_stored_values(
    node_id: u32,
    sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Ok(Json(get_measurements(
        db_conn,
        node_id,
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
            .map_err(|e| anyhow!("Failed to load list of nodes from DB. {e:?}"))?
    };

    let grouped_sensors: Vec<Vec<Sensor>> = {
        Sensor::belonging_to(&nodes)
            .load::<Sensor>(&*db_conn)
            .map_err(|e| anyhow!("Failed to load list of sensors from DB. {e:?}"))?
            .grouped_by(&nodes)
    };

    let nodes_and_sensors: Vec<(Node, Vec<Sensor>)> =
        nodes.into_iter().zip(grouped_sensors).collect();

    let mut output_map = HashMap::new();

    for (node, sensor_vec) in nodes_and_sensors {
        let node_map = output_map
            .entry(node.public_id as u32)
            .or_insert_with(HashMap::new);

        for sensor in sensor_vec {
            node_map
                .entry(sensor.sensor_type.as_ref().to_string())
                .or_insert_with(Vec::new)
                .push(sensor.public_id as u32);
        }
    }

    Ok(Json(output_map))
}

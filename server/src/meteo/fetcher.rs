use std::convert::TryInto;

use crate::db::models::Node;
use crate::db::DbConnPool;

use crate::meteo::models::Sensor;
use crate::meteo::node::SensorNodeRegistry;

use diesel::insert_into;
use diesel::prelude::*;

use crate::utils::Result;
use anyhow::anyhow;

use log::warn;

use crate::utils::DateTimeUtc;

pub fn fetcher_iteration(
    db_conn_pool: &DbConnPool,
    node_registry: &SensorNodeRegistry,
) -> Result<()> {
    let db = db_conn_pool
        .get()
        .map_err(|e| anyhow!("Failed to get DB connection. {e:?}"))?;

    // Get all sensors
    let sensors = {
        use crate::db::schema::*;
        use crate::meteo::schema::*;

        sensors::table
            .inner_join(nodes::table)
            .load::<(Sensor, Node)>(&db)
            .map_err(|e| anyhow!("{e:?}"))?
    };

    let curr_time = DateTimeUtc::now();

    for (ref sensor, ref node) in &sensors {
        // Send message querying each sensor
        let sens_id = sensor.public_id.try_into().unwrap(); // FIXME switch to map_err()?
        let node_id = node.public_id.try_into().unwrap();

        let measured_val = node_registry
            .get_node(node_id)?
            .measure(sensor.sensor_type, sens_id)?;

        // Push to db (use same timestamp for all values)
        {
            use crate::meteo::schema::measurements::dsl::*;

            if insert_into(measurements)
                .values((
                    sensor_id.eq(sensor.id),
                    value.eq(measured_val),
                    measured_at.eq(&curr_time),
                ))
                .execute(&db)
                .is_err()
            {
                warn!(
                    "Error while inserting measurement: (id {}, value {}, measured_at {:?})",
                    sensor.id, measured_val, curr_time
                );
            }
        }
    }

    Ok(())
}

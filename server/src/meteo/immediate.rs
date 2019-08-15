use rocket::State;
use rocket_contrib::json::Json;

use crate::comm;
use crate::db::Db;

use super::messages::{transfer, IncomingMessage, OutgoingMessage};
use super::{MeteoError, MeteoResponse};

use super::models::{Sensor, SensorTypeEnum};

use crate::utils::IdRange;

use log::warn;

use std::collections::HashMap;

use diesel::prelude::*;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorState {
    id: u32,
    #[serde(rename = "type")]
    sensor_type: String,
    #[serde(rename = "lastVal")]
    last_val: f32,
}

// FIXME Remove this endpoint during work on #15. Added only for initial version of ratfist-mobile.
#[get("/current")]
pub fn query_all_sensors(
    db_conn: Db,
    comm_state: State<'_, comm::CommState>,
) -> MeteoResponse<Vec<SensorState>> {
    let mut output = Vec::new();

    let sensors = {
        use crate::meteo::schema::*;

        sensors::table
            .load::<Sensor>(&*db_conn)
    }
    .map_err(|_| MeteoError)?;

    for sensor in &sensors {
        // Send message querying each sensor

        let sensor_type_enum = sensor.sensor_id;
        let sens_id = sensor.public_id as u32;

        let outgoing_msg = match sensor_type_enum {
            SensorTypeEnum::Pressure => OutgoingMessage::GetPressure(sens_id),
            SensorTypeEnum::Temperature => OutgoingMessage::GetTemperature(sens_id),
            SensorTypeEnum::Humidity => OutgoingMessage::GetHumidity(sens_id),
            SensorTypeEnum::LightLevel => OutgoingMessage::GetLightLevel(sens_id),
        };

        let channel = comm_state.get_comm_channel(0).map_err(|_| MeteoError)?;

        let measured_val = match transfer(&channel, outgoing_msg) {
            Ok(IncomingMessage::Pressure(id, val))
                if id == sens_id && sensor_type_enum == SensorTypeEnum::Pressure =>
            {
                val
            }
            Ok(IncomingMessage::Temperature(id, val))
                if id == sens_id && sensor_type_enum == SensorTypeEnum::Temperature =>
            {
                val
            }
            Ok(IncomingMessage::Humidity(id, val))
                if id == sens_id && sensor_type_enum == SensorTypeEnum::Humidity =>
            {
                val
            }
            Ok(IncomingMessage::LightLevel(id, val))
                if id == sens_id && sensor_type_enum == SensorTypeEnum::LightLevel =>
            {
                val
            }
            Ok(msg) => {
                warn!("Unexpected reply message: {:?}", msg);
                return Err(MeteoError);
            }
            Err(e) => {
                warn!("Communication error: {:?}", e);
                return Err(MeteoError);
            }
        };

        let sensor_type_str = match sensor_type_enum {
            SensorTypeEnum::Pressure => "Pressure",
            SensorTypeEnum::Temperature => "Temperature",
            SensorTypeEnum::Humidity => "Humidity",
            SensorTypeEnum::LightLevel => "Light Level",
        }
        .to_string();

        output.push(SensorState {
            id: sens_id,
            sensor_type: sensor_type_str,
            last_val: measured_val,
        });
    }

    Ok(Json(output))
}

#[get("/<node_ids>/<sensor_type>/<sensor_ids>", format = "application/json")]
pub fn query_current_values(
    node_ids: IdRange,
    sensor_type: SensorTypeEnum,
    sensor_ids: IdRange,
    comm_state: State<'_, comm::CommState>,
) -> MeteoResponse<HashMap<u32, HashMap<u32, f32>>> {
    let mut response_map = HashMap::new();

    for node_id in node_ids {
        let comm = comm_state
            .get_comm_channel(node_id)
            .map_err(|_| MeteoError)?;

        let node_map = response_map.entry(node_id).or_insert_with(HashMap::new);

        for sensor_id in sensor_ids.iter() {
            let outgoing_msg = match sensor_type {
                SensorTypeEnum::Pressure => OutgoingMessage::GetPressure(*sensor_id),
                SensorTypeEnum::Temperature => OutgoingMessage::GetTemperature(*sensor_id),
                SensorTypeEnum::Humidity => OutgoingMessage::GetHumidity(*sensor_id),
                SensorTypeEnum::LightLevel => OutgoingMessage::GetLightLevel(*sensor_id),
            };

            let incoming_msg = transfer(&comm, outgoing_msg)?;

            let val = match incoming_msg {
                IncomingMessage::Pressure(in_sensor_id, val)
                    if sensor_type == SensorTypeEnum::Pressure && in_sensor_id == *sensor_id =>
                {
                    val
                }
                IncomingMessage::Temperature(in_sensor_id, val)
                    if sensor_type == SensorTypeEnum::Temperature && in_sensor_id == *sensor_id =>
                {
                    val
                }
                IncomingMessage::Humidity(in_sensor_id, val)
                    if sensor_type == SensorTypeEnum::Humidity && in_sensor_id == *sensor_id =>
                {
                    val
                }
                IncomingMessage::LightLevel(in_sensor_id, val)
                    if sensor_type == SensorTypeEnum::LightLevel && in_sensor_id == *sensor_id =>
                {
                    val
                }
                _ => {
                    return Err(MeteoError);
                }
            };

            node_map.insert(*sensor_id, val);
        }
    }

    Ok(Json(response_map))
}

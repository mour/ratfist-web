use rocket::State;
use rocket_contrib::Json;

use comm;
use db::Db;

use super::messages::{transfer, IncomingMessage, OutgoingMessage};
use super::{MeteoError, MeteoResponse};

use super::models::{Sensor, SensorType, SensorTypeEnum};

use utils::IdRange;

use std::collections::HashMap;

use diesel::prelude::*;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
struct SensorState {
    id: u32,
    #[serde(rename = "type")]
    sensor_type: String,
    #[serde(rename = "lastVal")]
    last_val: f32,
}

// FIXME Remove this endpoint during work on #15. Added only for initial version of ratfist-mobile.
#[get("/current")]
fn query_all_sensors(db_conn: Db, comm_state: State<comm::CommState>) -> MeteoResponse<Vec<SensorState>> {

    let mut output = Vec::new();

    let sensors = {
        use meteo::schema::*;

        sensors::table
            .inner_join(sensor_types::table)
            .load::<(Sensor, SensorType)>(&*db_conn)
    }.map_err(|_| MeteoError)?;

    for (ref sensor, ref sensor_type) in &sensors {
        // Send message querying each sensor

        let sensor_type_enum = SensorTypeEnum::try_from(sensor_type.name.as_str())?;
        let sens_id = sensor.public_id as u32;

        let outgoing_msg = match sensor_type_enum {
            SensorTypeEnum::Pressure => OutgoingMessage::GetPressure(sens_id),
            SensorTypeEnum::Temperature => OutgoingMessage::GetTemperature(sens_id),
            SensorTypeEnum::Humidity => OutgoingMessage::GetHumidity(sens_id),
            SensorTypeEnum::LightLevel => OutgoingMessage::GetLightLevel(sens_id),
        };

        let channel = comm_state.get_comm_channel();

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
        }.to_string();

        output.push(SensorState {
            id: sens_id,
            sensor_type: sensor_type_str,
            last_val: measured_val,
        });
    }

    Ok(Json(output))
}


#[get("/<id_range>/pressure")]
fn query_current_pressure(
    id_range: IdRange,
    comm_state: State<comm::CommState>,
) -> Result<Json<HashMap<u32, f32>>, MeteoError> {
    let comm = comm_state.get_comm_channel();

    let mut press_map = HashMap::new();

    for id in id_range {
        let resp = transfer(&comm, OutgoingMessage::GetPressure(id))?;

        match resp {
            IncomingMessage::Pressure(ch, val) if ch == id => {
                press_map.insert(ch, val);
            }
            IncomingMessage::RetVal(_val) => {}
            _ => return Err(MeteoError),
        }
    }

    Ok(Json(press_map))
}

#[get("/<id_range>/temperature")]
fn query_current_temperature(
    id_range: IdRange,
    comm_state: State<comm::CommState>,
) -> Result<Json<HashMap<u32, f32>>, MeteoError> {
    let comm = comm_state.get_comm_channel();

    let mut press_map = HashMap::new();

    for id in id_range {
        let resp = transfer(&comm, OutgoingMessage::GetTemperature(id))?;

        match resp {
            IncomingMessage::Temperature(ch, val) if ch == id => {
                press_map.insert(ch, val);
            }
            IncomingMessage::RetVal(_val) => {}
            _ => return Err(MeteoError),
        }
    }

    Ok(Json(press_map))
}

#[get("/<id_range>/humidity")]
fn query_current_humidity(
    id_range: IdRange,
    comm_state: State<comm::CommState>,
) -> Result<Json<HashMap<u32, f32>>, MeteoError> {
    let comm = comm_state.get_comm_channel();

    let mut press_map = HashMap::new();

    for id in id_range {
        let resp = transfer(&comm, OutgoingMessage::GetHumidity(id))?;

        match resp {
            IncomingMessage::Humidity(ch, val) if ch == id => {
                press_map.insert(ch, val);
            }
            IncomingMessage::RetVal(_val) => {}
            _ => return Err(MeteoError),
        }
    }

    Ok(Json(press_map))
}

#[get("/<id_range>/light_level")]
fn query_current_light_level(
    id_range: IdRange,
    comm_state: State<comm::CommState>,
) -> Result<Json<HashMap<u32, f32>>, MeteoError> {
    let comm = comm_state.get_comm_channel();

    let mut press_map = HashMap::new();

    for id in id_range {
        let resp = transfer(&comm, OutgoingMessage::GetLightLevel(id))?;

        match resp {
            IncomingMessage::LightLevel(ch, val) if ch == id => {
                press_map.insert(ch, val);
            }
            IncomingMessage::RetVal(_val) => {}
            _ => return Err(MeteoError),
        }
    }

    Ok(Json(press_map))
}

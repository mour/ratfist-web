use rocket::State;
use rocket_contrib::Json;

use comm;

use super::messages::{transfer, IncomingMessage, OutgoingMessage};
use super::MeteoError;

use utils::IdRange;

use std::collections::HashMap;

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

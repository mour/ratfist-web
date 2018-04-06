
use rocket::State;
use rocket_contrib::Json;

use comm;

use meteo::MeteoError;

use utils::IdRange;

use std::str::FromStr;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub enum OutgoingMessage {
    GetPressure(u32),
    GetTemperature(u32),
    GetHumidity(u32),
    GetLightLevel(u32),
}

impl From<OutgoingMessage> for String {
    fn from(msg: OutgoingMessage) -> String {
        match msg {
            OutgoingMessage::GetPressure(ch) => format!("METEO,GET_PRESSURE,{}", ch),
            OutgoingMessage::GetTemperature(ch) => format!("METEO,GET_TEMPERATURE,{}", ch),
            OutgoingMessage::GetHumidity(ch) => format!("METEO,GET_HUMIDITY,{}", ch),
            OutgoingMessage::GetLightLevel(ch) => format!("METEO,GET_LIGHT_LEVEL,{}", ch),
        }
    }
}

#[derive(Debug)]
enum IncomingMessage {
    Pressure(u32, f32),
    Temperature(u32, f32),
    Humidity(u32, f32),
    LightLevel(u32, f32),
    RetVal(i32),
}

impl FromStr for IncomingMessage {
    type Err = MeteoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(',');
        if tokens.next() != Some("METEO") {
            return Err(MeteoError);
        }

        if let Some(msg_type) = tokens.next() {
            match msg_type {
                "PRESSURE_REPLY" => {
                    let ch_num = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;
                    let val = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;

                    Ok(IncomingMessage::Pressure(ch_num, val))
                }
                "TEMPERATURE_REPLY" => {
                    let ch_num = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;
                    let val = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;

                    Ok(IncomingMessage::Temperature(ch_num, val))
                }
                "HUMIDITY_REPLY" => {
                    let ch_num = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;
                    let val = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;

                    Ok(IncomingMessage::Humidity(ch_num, val))
                }
                "LIGHT_LEVEL_REPLY" => {
                    let ch_num = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;
                    let val = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;

                    Ok(IncomingMessage::LightLevel(ch_num, val))
                }
                "RET_VAL" => {
                    let ret_val = tokens.next().ok_or(MeteoError)?.parse().map_err(
                        |_| MeteoError,
                    )?;

                    Ok(IncomingMessage::RetVal(ret_val))
                }
                _ => Err(MeteoError),
            }
        } else {
            Err(MeteoError)
        }
    }
}


fn transfer(
    comm_ch: &comm::CommChannelTx,
    msg: OutgoingMessage,
) -> Result<IncomingMessage, MeteoError> {
    let msg_str = msg.into();
    debug!("Sending: {}", msg_str);

    let response_channel = comm_ch.send(msg_str).map_err(|e| {
        warn!("{}", e);
        MeteoError
    })?;

    let raw_response_msg = response_channel
        .recv_timeout(Duration::from_secs(3))
        .map_err(|e| {
            warn!("{}", e);
            MeteoError
        })?;

    debug!("Response: {}", raw_response_msg);

    raw_response_msg.parse()
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

use super::MeteoError;
use crate::comm;

use std::str::FromStr;
use std::time::Duration;

use log::{debug, warn};

#[derive(Debug)]
pub(super) enum OutgoingMessage {
    GetPressure(u32),
    GetTemperature(u32),
    GetHumidity(u32),
    GetLightLevel(u32),
}

impl From<&OutgoingMessage> for String {
    fn from(msg: &OutgoingMessage) -> String {
        match msg {
            OutgoingMessage::GetPressure(ch) => format!("METEO,GET_PRESSURE,{}", ch),
            OutgoingMessage::GetTemperature(ch) => format!("METEO,GET_TEMPERATURE,{}", ch),
            OutgoingMessage::GetHumidity(ch) => format!("METEO,GET_HUMIDITY,{}", ch),
            OutgoingMessage::GetLightLevel(ch) => format!("METEO,GET_LIGHT_LEVEL,{}", ch),
        }
    }
}

#[derive(Debug)]
pub(super) enum IncomingMessage {
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
                    let ch_num = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Pressure(ch_num, val))
                }
                "TEMPERATURE_REPLY" => {
                    let ch_num = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Temperature(ch_num, val))
                }
                "HUMIDITY_REPLY" => {
                    let ch_num = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Humidity(ch_num, val))
                }
                "LIGHT_LEVEL_REPLY" => {
                    let ch_num = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::LightLevel(ch_num, val))
                }
                "RET_VAL" => {
                    let ret_val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::RetVal(ret_val))
                }
                _ => Err(MeteoError),
            }
        } else {
            Err(MeteoError)
        }
    }
}

pub(super) fn transfer(
    comm_ch: &comm::CommChannelTx,
    msg: OutgoingMessage,
) -> Result<IncomingMessage, MeteoError> {
    let msg_str = (&msg).into();
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

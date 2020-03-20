use std::sync::{Arc, Mutex};

use crate::comm;
use crate::comm::serial::CommChannelTx;

use super::SensorNode;

use crate::meteo::models::SensorTypeEnum;
use crate::meteo::MeteoError;

use std::str::FromStr;
use std::time::Duration;

use log::{debug, warn};

#[allow(clippy::enum_variant_names)]
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
                    let node_id = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Pressure(node_id, val))
                }
                "TEMPERATURE_REPLY" => {
                    let node_id = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Temperature(node_id, val))
                }
                "HUMIDITY_REPLY" => {
                    let node_id = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::Humidity(node_id, val))
                }
                "LIGHT_LEVEL_REPLY" => {
                    let node_id = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;
                    let val = tokens
                        .next()
                        .ok_or(MeteoError)?
                        .parse()
                        .map_err(|_| MeteoError)?;

                    Ok(IncomingMessage::LightLevel(node_id, val))
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

pub struct SerialNode {
    node_public_id: u32,
    comm_channel: Arc<Mutex<CommChannelTx>>,
}

impl SerialNode {
    pub fn new(node_public_id: u32, serial_comm_path_id: u32) -> SerialNode {
        SerialNode {
            node_public_id,
            comm_channel: comm::get_serial_comm_path(serial_comm_path_id),
        }
    }

    fn transfer(&self, msg: OutgoingMessage) -> Result<IncomingMessage, MeteoError> {
        let msg_str = (&msg).into();
        debug!("Sending: {}", msg_str);

        let response_channel = self
            .comm_channel
            .lock()
            .expect("mutex poisoned")
            .send(self.node_public_id, msg_str)
            .map_err(|e| {
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
}

impl SensorNode for SerialNode {
    fn measure(&self, measurement_type: SensorTypeEnum, sensor_id: u32) -> Result<f32, MeteoError> {
        let outgoing_msg = match measurement_type {
            SensorTypeEnum::Pressure => OutgoingMessage::GetPressure(sensor_id),
            SensorTypeEnum::Temperature => OutgoingMessage::GetTemperature(sensor_id),
            SensorTypeEnum::Humidity => OutgoingMessage::GetHumidity(sensor_id),
            SensorTypeEnum::LightLevel => OutgoingMessage::GetLightLevel(sensor_id),
        };

        match self.transfer(outgoing_msg) {
            Ok(IncomingMessage::Pressure(id, val))
                if id == sensor_id && measurement_type == SensorTypeEnum::Pressure =>
            {
                Ok(val)
            }
            Ok(IncomingMessage::Temperature(id, val))
                if id == sensor_id && measurement_type == SensorTypeEnum::Temperature =>
            {
                Ok(val)
            }
            Ok(IncomingMessage::Humidity(id, val))
                if id == sensor_id && measurement_type == SensorTypeEnum::Humidity =>
            {
                Ok(val)
            }
            Ok(IncomingMessage::LightLevel(id, val))
                if id == sensor_id && measurement_type == SensorTypeEnum::LightLevel =>
            {
                Ok(val)
            }
            Ok(msg) => {
                warn!("Unexpected reply message: {:?}", msg);
                Err(MeteoError)
            }
            Err(e) => {
                warn!("Communication error: {:?}", e);
                Err(MeteoError)
            }
        }
    }
}

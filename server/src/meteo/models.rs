use super::schema::{measurements, sensor_types, sensors};
use super::MeteoError;

use utils::DateTimeUtc;

use std::borrow::Borrow;
use std::convert::TryFrom;

#[derive(Identifiable, Queryable, Debug, Clone)]
pub(super) struct Node {
    pub id: i32,
    pub public_id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Debug, Clone)]
pub(super) struct Sensor {
    pub id: i32,
    pub public_id: i32,
    pub node_id: i32,
    pub type_id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, Debug, Clone)]
#[belongs_to(Sensor)]
pub(super) struct Measurement {
    pub id: i32,
    pub sensor_id: i32,
    pub value: f32,
    pub measured_at: DateTimeUtc,
}

#[derive(Identifiable, Queryable, Debug, Clone)]
pub(super) struct SensorType {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(super) enum SensorTypeEnum {
    Pressure,
    Temperature,
    Humidity,
    LightLevel,
}

impl Borrow<str> for SensorTypeEnum {
    fn borrow(&self) -> &'static str {
        match self {
            SensorTypeEnum::Pressure => "pressure",
            SensorTypeEnum::Temperature => "temperature",
            SensorTypeEnum::Humidity => "humidity",
            SensorTypeEnum::LightLevel => "light level",
        }
    }
}

impl<'a> TryFrom<&'a str> for SensorTypeEnum {
    type Error = MeteoError;

    fn try_from(sensor_type_str: &'a str) -> Result<Self, Self::Error> {
        match sensor_type_str {
            "pressure" => Ok(SensorTypeEnum::Pressure),
            "temperature" => Ok(SensorTypeEnum::Temperature),
            "humidity" => Ok(SensorTypeEnum::Humidity),
            "light level" => Ok(SensorTypeEnum::LightLevel),
            _ => Err(MeteoError),
        }
    }
}

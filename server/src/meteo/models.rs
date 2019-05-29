use rocket::http::RawStr;
use rocket::request::FromParam;

use super::schema::{measurements, sensor_types, sensors};
use super::MeteoError;

use crate::utils::DateTimeUtc;

use std::borrow::Borrow;
use std::convert::TryFrom;

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
pub enum SensorTypeEnum {
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
            SensorTypeEnum::LightLevel => "light_level",
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
            "light_level" => Ok(SensorTypeEnum::LightLevel),
            _ => Err(MeteoError),
        }
    }
}

impl<'req> FromParam<'req> for SensorTypeEnum {
    type Error = MeteoError;

    fn from_param(param: &'req RawStr) -> Result<Self, Self::Error> {
        SensorTypeEnum::try_from(param.as_str())
    }
}

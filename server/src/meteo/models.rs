use rocket::http::RawStr;
use rocket::request::FromParam;

use diesel::backend::Backend;
use diesel::types::FromSql;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;

use super::schema::{measurements, sensors};
use super::MeteoError;

use crate::db::models::Node;

use crate::utils::DateTimeUtc;

use std::borrow::Borrow;
use std::convert::TryFrom;
use std::error::Error;

#[derive(Identifiable, Queryable, Associations, Debug, Clone)]
#[belongs_to(Node)]
pub struct Sensor {
    pub id: i32,
    pub public_id: i32,
    pub node_id: i32,
    pub type_id: SensorTypeEnum,
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

// #[derive(Identifiable, Queryable, Debug, Clone)]
// pub struct SensorType {
//     pub id: i32,
//     pub name: String,
// }

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

impl FromSql<Integer, Sqlite> for SensorTypeEnum {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> Result<SensorTypeEnum, Box<dyn Error + Send + Sync>> {

        let _val = i32::from_sql(bytes)?;
        Ok(SensorTypeEnum::Humidity)
    }
}

impl<'req> FromParam<'req> for SensorTypeEnum {
    type Error = MeteoError;

    fn from_param(param: &'req RawStr) -> Result<Self, Self::Error> {
        SensorTypeEnum::try_from(param.as_str())
    }
}

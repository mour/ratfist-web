use rocket::http::RawStr;
use rocket::request::FromParam;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Integer;

use super::schema::{measurements, sensors};
use super::MeteoError;

use crate::db::models::Node;

use crate::utils::DateTimeUtc;

use std::convert::TryFrom;
use std::io::Write;

#[derive(Identifiable, Queryable, Associations, Debug, Clone)]
#[belongs_to(Node)]
pub struct Sensor {
    pub id: i32,
    pub public_id: i32,
    pub node_id: i32,
    pub sensor_type: SensorTypeEnum,
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromSqlRow, AsExpression)]
#[sql_type = "Integer"]
#[repr(i32)]
pub enum SensorTypeEnum {
    Pressure = 0,
    Temperature = 1,
    Humidity = 2,
    LightLevel = 3,
}

impl AsRef<str> for SensorTypeEnum {
    fn as_ref(&self) -> &'static str {
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

impl<DB> FromSql<Integer, DB> for SensorTypeEnum
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&<DB as Backend>::RawValue>) -> deserialize::Result<Self> {
        let raw_val = i32::from_sql(bytes)?;
        match raw_val {
            x if x == SensorTypeEnum::Pressure as i32 => Ok(SensorTypeEnum::Pressure),
            x if x == SensorTypeEnum::Temperature as i32 => Ok(SensorTypeEnum::Temperature),
            x if x == SensorTypeEnum::Humidity as i32 => Ok(SensorTypeEnum::Humidity),
            x if x == SensorTypeEnum::LightLevel as i32 => Ok(SensorTypeEnum::LightLevel),
            _ => Err(Box::new(MeteoError)),
        }
    }
}

impl<DB> ToSql<Integer, DB> for SensorTypeEnum
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

impl<'req> FromParam<'req> for SensorTypeEnum {
    type Error = MeteoError;

    fn from_param(param: &'req RawStr) -> Result<Self, Self::Error> {
        SensorTypeEnum::try_from(param.as_str())
    }
}

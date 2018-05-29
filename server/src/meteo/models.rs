use super::schema::{measurements, sensor_types, sensors};

use utils::DateTimeUtc;

#[derive(Identifiable, Queryable, Debug)]
pub struct Sensor {
    pub id: i32,
    pub public_id: i32,
    pub type_id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[belongs_to(Sensor)]
pub struct Measurement {
    pub id: i32,
    pub sensor_id: i32,
    pub value: f32,
    pub measured_at: DateTimeUtc,
}

#[derive(Identifiable, Queryable, Debug)]
pub struct SensorType {
    pub id: i32,
    pub name: String,
}

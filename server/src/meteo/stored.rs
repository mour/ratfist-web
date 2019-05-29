use rocket_contrib::json::Json;

use crate::meteo::models::{Measurement, Sensor, SensorType, SensorTypeEnum};
use crate::meteo::{MeteoError, MeteoResponse};

use crate::utils::{DateTimeUtc, IdRange, TimeRangeOptionalEndTime};

use std::borrow::Borrow;
use std::collections::HashMap;

use crate::db::Db;

use diesel::prelude::*;
use diesel::ExpressionMethods;

fn get_measurements(
    db_conn: &Db,
    sensor_type: SensorTypeEnum,
    sensor_ids: &IdRange,
    time_range: &TimeRangeOptionalEndTime,
) -> Result<HashMap<u32, Vec<(DateTimeUtc, f32)>>, MeteoError> {
    let ids = sensor_ids.iter().map(|v| *v as i32).collect::<Vec<i32>>();

    let sensor_type_id = {
        use crate::meteo::schema::sensor_types::dsl::*;

        let sensor_type_str: &str = sensor_type.borrow();
        sensor_types
            .filter(name.eq(sensor_type_str))
            .first::<SensorType>(&**db_conn)
            .map_err(|_| MeteoError)?
            .id
    };

    let sensor_query = {
        use crate::meteo::schema::sensors::dsl::*;

        sensors.filter(public_id.eq_any(ids).and(type_id.eq(sensor_type_id)))
    };

    let sensors = sensor_query
        .load::<Sensor>(&**db_conn)
        .map_err(|_| MeteoError)?;

    let now = DateTimeUtc::now();
    let measurement_query = {
        use crate::meteo::schema::measurements::dsl::*;

        Measurement::belonging_to(&sensors)
            .order_by(measured_at.asc())
            .filter(measured_at.ge(&time_range.from))
            .filter(measured_at.le(time_range.to.as_ref().unwrap_or(&now)))
    };

    let measurements = measurement_query
        .load::<Measurement>(&**db_conn)
        .map_err(|_| MeteoError)?;

    let mut output_map = HashMap::new();

    for m in measurements {
        let sensor_vals = output_map
            .entry(m.sensor_id as u32)
            .or_insert_with(Vec::new);
        sensor_vals.push((m.measured_at, m.value));
    }

    Ok(output_map)
}

#[get("/<id_range>/pressure?<from>&<to>", format = "application/json")]
pub fn get_stored_pressure(
    id_range: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Ok(Json(get_measurements(
        &db_conn,
        SensorTypeEnum::Pressure,
        &id_range,
        &TimeRangeOptionalEndTime { from, to },
    )?))
}

#[get("/<id_range>/temperature?<from>&<to>", format = "application/json")]
pub fn get_stored_temperature(
    id_range: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Ok(Json(get_measurements(
        &db_conn,
        SensorTypeEnum::Temperature,
        &id_range,
        &TimeRangeOptionalEndTime { from, to },
    )?))
}

#[get("/<id_range>/humidity?<from>&<to>", format = "application/json")]
pub fn get_stored_humidity(
    id_range: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Ok(Json(get_measurements(
        &db_conn,
        SensorTypeEnum::Humidity,
        &id_range,
        &TimeRangeOptionalEndTime { from, to },
    )?))
}

#[get("/<id_range>/light_level?<from>&<to>", format = "application/json")]
pub fn get_stored_light_level(
    id_range: IdRange,
    from: DateTimeUtc,
    to: Option<DateTimeUtc>,
    db_conn: Db,
) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Ok(Json(get_measurements(
        &db_conn,
        SensorTypeEnum::LightLevel,
        &id_range,
        &TimeRangeOptionalEndTime { from, to },
    )?))
}

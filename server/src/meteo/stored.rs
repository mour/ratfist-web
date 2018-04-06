
use rocket::State;
use rocket_contrib::Json;

use meteo::{MeteoResponse, MeteoError};
use utils::{IdRange, TimeRangeOptionalEndTime, DateTimeUtc};

use std::collections::HashMap;


#[get("/<id_range>/pressure?<time_range>", format = "application/json")]
fn get_stored_pressure(id_range: IdRange, time_range: TimeRangeOptionalEndTime) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Err(MeteoError)
}

#[get("/<id_range>/temperature?<time_range>", format = "application/json")]
fn get_stored_temperature(id_range: IdRange, time_range: TimeRangeOptionalEndTime) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Err(MeteoError)
}

#[get("/<id_range>/humidity?<time_range>", format = "application/json")]
fn get_stored_humidity(id_range: IdRange, time_range: TimeRangeOptionalEndTime) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Err(MeteoError)
}

#[get("/<id_range>/light_level?<time_range>", format = "application/json")]
fn get_stored_light_level(id_range: IdRange, time_range: TimeRangeOptionalEndTime) -> MeteoResponse<HashMap<u32, Vec<(DateTimeUtc, f32)>>> {
    Err(MeteoError)
}

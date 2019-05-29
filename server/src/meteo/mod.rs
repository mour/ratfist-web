use rocket::Route;
use rocket_contrib::json::Json;

pub mod fetcher;
mod immediate;
mod messages;
mod models;
mod schema;
mod stored;

#[derive(Debug)]
pub struct MeteoError;

type MeteoResponse<T> = Result<Json<T>, MeteoError>;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_all_sensors,
        immediate::query_current_value,
        stored::get_stored_pressure,
        stored::get_stored_temperature,
        stored::get_stored_humidity,
        stored::get_stored_light_level,
    ]
}

use rocket::Route;
use rocket_contrib::Json;

pub mod fetcher;
mod immediate;
mod messages;
mod models;
mod schema;
mod stored;

#[derive(Debug)]
struct MeteoError;

type MeteoResponse<T> = Result<Json<T>, MeteoError>;

pub fn get_routes() -> Vec<Route> {
    routes![
        immediate::query_current_pressure,
        immediate::query_current_temperature,
        immediate::query_current_humidity,
        immediate::query_current_light_level,
        stored::get_stored_pressure,
        stored::get_stored_temperature,
        stored::get_stored_humidity,
        stored::get_stored_light_level,
    ]
}
